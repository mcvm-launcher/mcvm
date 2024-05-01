use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use mcvm_core::io::json_to_file;
use mcvm_shared::id::InstanceRef;
use mcvm_shared::util::utc_timestamp;
use rand::Rng;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use zip::{ZipArchive, ZipWriter};

/// Name of the backup index file
pub const INDEX_NAME: &str = "index.json";
/// ID of the default group
pub const DEFAULT_GROUP: &str = "default";

/// Settings for backups
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct Config {
	/// Default settings for backups
	#[serde(flatten)]
	pub common: CommonConfig,
	/// backup groups
	pub groups: HashMap<String, GroupConfig>,
}

impl Config {
	/// Get the consolidated, final config for a group
	pub fn get_group_config(&self, group_id: &str) -> anyhow::Result<GroupConfig> {
		let mut out = GroupConfig {
			common: self.common.clone(),
			..Default::default()
		};

		if group_id == DEFAULT_GROUP {
			return Ok(out);
		};

		let Some(group) = self.groups.get(group_id) else {
			bail!("Group does not exist");
		};
		let group = group.clone();

		out.on = out.on.or(group.on);

		Ok(out)
	}
}

/// Configuration for a group of backups
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct GroupConfig {
	/// When the backup should be automatically created
	pub on: Option<BackupAutoHook>,
	/// backup settings for this group
	#[serde(flatten)]
	pub common: CommonConfig,
}

/// General configuration for backups and backup groups
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct CommonConfig {
	/// The max number of backups
	pub max_count: Option<u32>,
	/// The files and directories to include in the backup
	pub paths: Vec<String>,
	/// How the backup should be stored
	pub storage_type: StorageType,
}

/// When a backup should be automatically created
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
#[serde(rename_all = "snake_case")]
pub enum BackupAutoHook {
	/// Create a backup whenever the instance is launched
	OnLaunch,
	/// Create a backup whenever the instance is stopped
	OnStop,
}

/// Index for the backups of an instance
pub struct Index {
	/// The contents of the index
	pub contents: IndexContents,
	/// The path where the backups are
	pub dir: PathBuf,
	/// The config for the backups
	pub config: Config,
	/// The instance ref for this index
	pub inst_ref: InstanceRef,
}

/// Contents for the backup index
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct IndexContents {
	/// The list of available groups
	pub groups: HashMap<String, GroupEntry>,
}

impl Index {
	/// Gets the index path
	fn get_path(backup_directory: &Path) -> PathBuf {
		backup_directory.join(INDEX_NAME)
	}

	/// Open the index
	pub fn open(
		backup_directory: &Path,
		inst_ref: InstanceRef,
		config: &Config,
	) -> anyhow::Result<Self> {
		fs::create_dir_all(backup_directory)?;
		let path = Self::get_path(backup_directory);
		let contents = if path.exists() {
			let mut file = File::open(&path).context("Failed to open backup index")?;
			serde_json::from_reader(&mut file).context("Failed to parse JSON")?
		} else {
			IndexContents::default()
		};
		let index = Self {
			contents,
			dir: backup_directory.to_owned(),
			inst_ref,
			config: config.clone(),
		};

		Ok(index)
	}

	/// Finish using the index
	pub fn finish(&self) -> anyhow::Result<()> {
		let path = Self::get_path(&self.dir);
		json_to_file(path, &self.contents)?;

		Ok(())
	}

	/// Get a backup
	pub fn get_backup(&self, group_id: &str, backup_id: &str) -> anyhow::Result<&Entry> {
		self.contents
			.groups
			.get(group_id)
			.context("Group does not exist")?
			.backups
			.iter()
			.find(|x| x.id == backup_id)
			.context("Backup does not exist")
	}

	/// Create a new backup
	pub fn create_backup(
		&mut self,
		source: BackupSource,
		group_id: Option<&str>,
		instance_dir: &Path,
	) -> anyhow::Result<()> {
		let group_id = group_id.unwrap_or(DEFAULT_GROUP);

		let group_config = self.config.get_group_config(group_id)?;

		let backup_id = generate_random_id();
		let backup_path =
			self.get_backup_path(group_id, &backup_id, group_config.common.storage_type);

		let mut readers = Vec::new();
		for path in &group_config.common.paths {
			let paths = get_instance_file_paths(path, instance_dir)
				.context("Failed to get recursive file paths")?;
			for path in paths {
				let file = File::open(instance_dir.join(&path))
					.with_context(|| format!("Failed to open backed up file with path {path}"))?;
				let file = BufReader::new(file);
				readers.push((path.clone(), file));
			}
		}
		write_backup_files(&backup_path, &group_config, readers)?;

		let now = utc_timestamp()?;
		// Add the backup entry to the group
		let group_entry = self.contents.groups.entry(group_id.into()).or_default();
		group_entry.backups.push(Entry {
			id: backup_id,
			date: now,
			source,
			storage_type: group_config.common.storage_type,
		});

		self.remove_old_backups(group_id, &group_config)?;

		Ok(())
	}

	/// Remove a backup
	pub fn remove_backup(&mut self, group_id: &str, backup_id: &str) -> anyhow::Result<()> {
		let group_entry = self.contents.groups.entry(group_id.into()).or_default();
		let index = group_entry
			.backups
			.iter()
			.position(|x| x.id == backup_id)
			.ok_or(anyhow!("Backup with ID was not found"))?;
		let backup = &group_entry.backups[index];
		let storage_type = backup.storage_type;

		group_entry.backups.remove(index);

		let backup_path = self.get_backup_path(group_id, backup_id, storage_type);
		if backup_path.exists() {
			match storage_type {
				StorageType::Archive => fs::remove_file(backup_path)?,
				StorageType::Folder => fs::remove_dir_all(backup_path)?,
			}
		}

		Ok(())
	}

	/// Remove old backups that are over the limit
	pub fn remove_old_backups(
		&mut self,
		group_id: &str,
		group_config: &GroupConfig,
	) -> anyhow::Result<()> {
		let Some(group_entry) = self.contents.groups.get(group_id) else {
			return Ok(());
		};
		if let Some(limit) = group_config.common.max_count {
			if group_entry.backups.len() > (limit as usize) {
				let num_to_remove = group_entry.backups.len() - (limit as usize);
				let to_remove: Vec<String> = group_entry.backups[0..num_to_remove - 1]
					.iter()
					.map(|x| x.id.clone())
					.collect();
				for id in to_remove {
					self.remove_backup(group_id, &id)
						.with_context(|| format!("Failed to remove old backup '{id}'"))?;
				}
			}
		}

		Ok(())
	}

	/// Restores a backup
	pub fn restore_backup(
		&self,
		group_id: &str,
		backup_id: &str,
		instance_dir: &Path,
	) -> anyhow::Result<()> {
		let group_entry = self
			.contents
			.groups
			.get(group_id)
			.context("Group does not exist")?;
		let backup = group_entry
			.backups
			.iter()
			.find(|x| x.id == backup_id)
			.ok_or(anyhow!("Backup with ID was not found"))?;

		let backup_path = self.get_backup_path(group_id, backup_id, backup.storage_type);
		restore_backup_files(&backup_path, backup.storage_type, instance_dir)?;

		Ok(())
	}

	/// Gets the backup directory for a group
	fn get_group_dir(&self, group_id: &str) -> PathBuf {
		self.dir.join(group_id)
	}

	/// Get the path to a specific backup
	pub fn get_backup_path(
		&self,
		group_id: &str,
		backup_id: &str,
		storage_type: StorageType,
	) -> PathBuf {
		let path = self.get_group_dir(group_id);
		let filename = match storage_type {
			StorageType::Archive => format!("{backup_id}.zip"),
			StorageType::Folder => backup_id.to_owned(),
		};

		path.join(filename)
	}
}

/// A group entry in the backup index
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GroupEntry {
	/// The backups in this group
	pub backups: Vec<Entry>,
}

/// Entry for a backup in the backup index
#[derive(Serialize, Deserialize)]
pub struct Entry {
	/// The ID of the backup
	pub id: String,
	/// The timestamp when the backup was created
	pub date: u64,
	/// What kind of backup this is
	#[serde(alias = "kind")]
	pub source: BackupSource,
	/// How the backup is stored on the filesystem
	pub storage_type: StorageType,
}

/// Where a backup was created from
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupSource {
	/// A backup created by the user
	User,
	/// An automatically created backup
	Auto,
}

/// Format for stored backups
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum StorageType {
	/// Stored as normal in a new directory
	Folder,
	/// Packed into an archive format to save space
	#[default]
	Archive,
}

/// Get the backup directory for an instance
pub fn get_backup_directory(base_dir: &Path, inst_ref: &InstanceRef) -> PathBuf {
	base_dir
		.join(inst_ref.profile.to_string())
		.join(inst_ref.instance.to_string())
}

/// Generates a random backup ID
pub fn generate_random_id() -> String {
	let mut rng = rand::thread_rng();
	let num = rng.gen_range(0..std::u64::MAX);
	format!("{num:x}")
}

/// Gets all file paths from a user-provided path recursively
fn get_instance_file_paths(path: &str, instance_dir: &Path) -> anyhow::Result<Vec<String>> {
	// Handle glob patterns
	if path.contains('*') {
		let glob = format!("{}/{path}", instance_dir.to_string_lossy());
		let glob = glob::glob(&glob);

		if let Ok(glob) = glob {
			let mut out = Vec::new();
			for path in glob {
				let path = path?;
				let rel = path.strip_prefix(instance_dir)?;
				if path.is_dir() {
					let rel_str = format!("{}", rel.to_string_lossy());
					let recursive_paths = get_instance_file_paths(&rel_str, instance_dir)
						.context("Failed to read subdirectory")?;
					out.extend(recursive_paths);
				} else {
					out.push(rel.to_string_lossy().to_string());
				}
			}

			return Ok(out);
		}
	}

	let instance_path = instance_dir.join(path);
	if instance_path.is_file() {
		Ok(vec![path.to_owned()])
	} else {
		let mut paths = Vec::new();
		for entry in fs::read_dir(&instance_path)? {
			let entry = entry?;
			let sub_path = entry.path();
			let rel = sub_path.strip_prefix(instance_dir)?;
			let rel_str = rel.to_string_lossy().to_string();
			if sub_path.is_dir() {
				let recursive_paths = get_instance_file_paths(&rel_str, instance_dir)
					.context("Failed to read subdirectory")?;
				paths.extend(recursive_paths);
			} else {
				paths.push(rel_str);
			}
		}

		Ok(paths)
	}
}

/// Writes backup files to the stored format. Takes the path to the backup file / directory.
/// Readers are pairs of relative file paths and readers for files.
fn write_backup_files<R: Read>(
	backup_path: &Path,
	group_config: &GroupConfig,
	readers: Vec<(String, R)>,
) -> anyhow::Result<()> {
	match &group_config.common.storage_type {
		StorageType::Archive => {
			mcvm_core::io::files::create_leading_dirs(backup_path)?;
			let file = File::create(backup_path).context("Failed to create archive file")?;
			let mut file = BufWriter::new(file);
			let mut arc = ZipWriter::new(&mut file);
			let options = zip::write::FileOptions::default()
				.compression_method(zip::CompressionMethod::Deflated);
			for (path, mut reader) in readers {
				arc.start_file(path, options)?;
				std::io::copy(&mut reader, &mut arc).context("Failed to copy to archive file")?;
			}

			arc.finish()?;
		}
		StorageType::Folder => {
			for (path, mut reader) in readers {
				let dest = backup_path.join(path);
				mcvm_core::io::files::create_leading_dirs(&dest)?;
				let file =
					File::create(dest).context("Failed to create snapshot destination file")?;
				let mut file = BufWriter::new(file);
				std::io::copy(&mut reader, &mut file)
					.context("Failed to copy to snapshot destination file")?;
			}
		}
	};

	Ok(())
}

/// Restores backup files to the instance. Takes the path to the backup file / directory.
fn restore_backup_files(
	backup_path: &Path,
	storage_type: StorageType,
	instance_dir: &Path,
) -> anyhow::Result<()> {
	match storage_type {
		StorageType::Archive => {
			let file = File::open(backup_path)?;
			let mut file = BufReader::new(file);
			let mut arc = ZipArchive::new(&mut file)?;
			arc.extract(instance_dir)
				.context("Failed to extract backup archive")?;
		}
		StorageType::Folder => {
			mcvm_core::io::files::copy_dir_contents(backup_path, instance_dir)
				.context("Failed to copy directory")?;
		}
	}

	Ok(())
}
