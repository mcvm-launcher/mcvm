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

/// Name of the snapshot index file
pub const INDEX_NAME: &str = "index.json";
/// ID of the default group
pub const DEFAULT_GROUP: &str = "default";

/// Settings for snapshots
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct Config {
	/// Default settings for snapshots
	#[serde(flatten)]
	pub common: CommonConfig,
	/// Snapshot groups
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

/// Configuration for a group of snapshots
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct GroupConfig {
	/// When the snapshot should be automatically created
	pub on: Option<SnapshotAutoHook>,
	/// Snapshot settings for this group
	#[serde(flatten)]
	pub common: CommonConfig,
}

/// General configuration for snapshots and snapshot groups
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct CommonConfig {
	/// The max number of snapshots
	pub max_count: Option<u32>,
	/// The files and directories to include in the snapshot
	pub paths: Vec<String>,
	/// How the snapshot should be stored
	pub storage_type: StorageType,
}

/// When a snapshot should be automatically created
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SnapshotAutoHook {}

/// Index for the snapshots of an instance
pub struct Index {
	/// The contents of the index
	pub contents: IndexContents,
	/// The path where the snapshots are
	pub dir: PathBuf,
	/// The config for the snapshots
	pub config: Config,
	/// The instance ref for this index
	pub inst_ref: InstanceRef,
}

/// Contents for the snapshot index
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct IndexContents {
	/// The list of available groups
	pub groups: HashMap<String, GroupEntry>,
}

impl Index {
	/// Gets the index path
	fn get_path(snapshot_directory: &Path) -> PathBuf {
		snapshot_directory.join(INDEX_NAME)
	}

	/// Open the index
	pub fn open(
		snapshot_directory: &Path,
		inst_ref: InstanceRef,
		config: &Config,
	) -> anyhow::Result<Self> {
		fs::create_dir_all(snapshot_directory)?;
		let path = Self::get_path(snapshot_directory);
		let contents = if path.exists() {
			let mut file = File::open(&path).context("Failed to open snapshot index")?;
			serde_json::from_reader(&mut file).context("Failed to parse JSON")?
		} else {
			IndexContents::default()
		};
		let index = Self {
			contents,
			dir: snapshot_directory.to_owned(),
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

	/// Get a snapshot
	pub fn get_snapshot(&self, group_id: &str, snapshot_id: &str) -> anyhow::Result<&Entry> {
		self.contents
			.groups
			.get(group_id)
			.context("Group does not exist")?
			.snapshots
			.iter()
			.find(|x| x.id == snapshot_id)
			.context("Snapshot does not exist")
	}

	/// Create a new snapshot
	pub fn create_snapshot(
		&mut self,
		kind: SnapshotKind,
		group_id: Option<&str>,
		instance_dir: &Path,
	) -> anyhow::Result<()> {
		let group_id = group_id.unwrap_or(DEFAULT_GROUP);
		// // Remove the snapshot if it exists already
		// if self.snapshot_exists(&snapshot_id) {
		// 	self.remove_snapshot(&snapshot_id)
		// 		.context("Failed to remove existing snapshot with same ID")?;
		// }

		let group_config = self.config.get_group_config(group_id)?;

		let snapshot_id = generate_random_id();
		let snapshot_path =
			self.get_snapshot_path(group_id, &snapshot_id, group_config.common.storage_type);

		let mut readers = Vec::new();
		for path in &group_config.common.paths {
			let paths = get_instance_file_paths(path, instance_dir)
				.context("Failed to get recursive file paths")?;
			for path in paths {
				let file = File::open(instance_dir.join(&path))
					.with_context(|| format!("Failed to open snapshotted file with path {path}"))?;
				let file = BufReader::new(file);
				readers.push((path.clone(), file));
			}
		}
		write_snapshot_files(&snapshot_path, &group_config, readers)?;

		let now = utc_timestamp()?;
		// Add the snapshot entry to the group
		let group_entry = self
			.contents
			.groups
			.entry(group_id.into())
			.or_insert_with(GroupEntry::default);
		group_entry.snapshots.push(Entry {
			id: snapshot_id,
			date: now,
			kind,
			storage_type: group_config.common.storage_type,
		});

		self.remove_old_snapshots(group_id, &group_config)?;

		Ok(())
	}

	/// Remove a snapshot
	pub fn remove_snapshot(&mut self, group_id: &str, snapshot_id: &str) -> anyhow::Result<()> {
		let group_entry = self
			.contents
			.groups
			.entry(group_id.into())
			.or_insert_with(GroupEntry::default);
		let index = group_entry
			.snapshots
			.iter()
			.position(|x| x.id == snapshot_id)
			.ok_or(anyhow!("Snapshot with ID was not found"))?;
		let snapshot = &group_entry.snapshots[index];
		let storage_type = snapshot.storage_type;

		group_entry.snapshots.remove(index);

		let snapshot_path = self.get_snapshot_path(group_id, snapshot_id, storage_type);
		if snapshot_path.exists() {
			match storage_type {
				StorageType::Archive => fs::remove_file(snapshot_path)?,
				StorageType::Folder => fs::remove_dir_all(snapshot_path)?,
			}
		}

		Ok(())
	}

	/// Remove old snapshots that are over the limit
	pub fn remove_old_snapshots(
		&mut self,
		group_id: &str,
		group_config: &GroupConfig,
	) -> anyhow::Result<()> {
		let Some(group_entry) = self.contents.groups.get(group_id) else {
			return Ok(());
		};
		if let Some(limit) = group_config.common.max_count {
			if group_entry.snapshots.len() > (limit as usize) {
				let num_to_remove = group_entry.snapshots.len() - (limit as usize);
				let to_remove: Vec<String> = group_entry.snapshots[0..num_to_remove - 1]
					.iter()
					.map(|x| x.id.clone())
					.collect();
				for id in to_remove {
					self.remove_snapshot(group_id, &id)
						.with_context(|| format!("Failed to remove old snapshot '{id}'"))?;
				}
			}
		}

		Ok(())
	}

	/// Restores a snapshot
	pub fn restore_snapshot(
		&self,
		group_id: &str,
		snapshot_id: &str,
		instance_dir: &Path,
	) -> anyhow::Result<()> {
		let group_entry = self
			.contents
			.groups
			.get(group_id)
			.context("Group does not exist")?;
		let snapshot = group_entry
			.snapshots
			.iter()
			.find(|x| x.id == snapshot_id)
			.ok_or(anyhow!("Snapshot with ID was not found"))?;

		let snapshot_path = self.get_snapshot_path(group_id, snapshot_id, snapshot.storage_type);
		restore_snapshot_files(&snapshot_path, snapshot.storage_type, instance_dir)?;

		Ok(())
	}

	/// Gets the snapshot directory for a group
	fn get_group_dir(&self, group_id: &str) -> PathBuf {
		self.dir.join(group_id)
	}

	/// Get the path to a specific snapshot
	pub fn get_snapshot_path(
		&self,
		group_id: &str,
		snapshot_id: &str,
		storage_type: StorageType,
	) -> PathBuf {
		let path = self.get_group_dir(group_id);
		let filename = match storage_type {
			StorageType::Archive => format!("{snapshot_id}.zip"),
			StorageType::Folder => snapshot_id.to_owned(),
		};

		path.join(filename)
	}
}

/// A group entry in the snapshot index
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GroupEntry {
	/// The snapshots in this group
	pub snapshots: Vec<Entry>,
}

/// Entry for a snapshot in the snapshot index
#[derive(Serialize, Deserialize)]
pub struct Entry {
	/// The ID of the snapshot
	pub id: String,
	/// The timestamp when the snapshot was created
	pub date: u64,
	/// What kind of snapshot this is
	pub kind: SnapshotKind,
	/// How the snapshot is stored on the filesystem
	pub storage_type: StorageType,
}

/// Type of a snapshot
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
	/// A snapshot created by the user
	User,
}

/// Format for stored snapshots
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

/// Get the snapshot directory for an instance
pub fn get_snapshot_directory(base_dir: &Path, inst_ref: &InstanceRef) -> PathBuf {
	base_dir
		.join(inst_ref.profile.to_string())
		.join(inst_ref.instance.to_string())
}

/// Generates a random snapshot ID
pub fn generate_random_id() -> String {
	let mut rng = rand::thread_rng();
	let num = rng.gen_range(0..std::u64::MAX);
	format!("{num:x}")
}

/// Gets all file paths from a user-provided path recursively
fn get_instance_file_paths(path: &str, instance_dir: &Path) -> anyhow::Result<Vec<String>> {
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

/// Writes snapshot files to the stored format. Takes the path to the snapshot file / directory.
/// Readers are pairs of relative file paths and readers for files.
fn write_snapshot_files<R: Read>(
	snapshot_path: &Path,
	group_config: &GroupConfig,
	readers: Vec<(String, R)>,
) -> anyhow::Result<()> {
	match &group_config.common.storage_type {
		StorageType::Archive => {
			let file = File::create(snapshot_path)?;
			let mut file = BufWriter::new(file);
			let mut arc = ZipWriter::new(&mut file);
			let options = zip::write::FileOptions::default()
				.compression_method(zip::CompressionMethod::Deflated);
			for (path, mut reader) in readers {
				arc.start_file(path, options)?;
				std::io::copy(&mut reader, &mut arc)?;
			}

			arc.finish()?;
		}
		StorageType::Folder => {
			for (path, mut reader) in readers {
				let dest = snapshot_path.join(path);
				mcvm_core::io::files::create_leading_dirs(&dest)?;
				let file = File::create(dest)?;
				let mut file = BufWriter::new(file);
				std::io::copy(&mut reader, &mut file)?;
			}
		}
	};

	Ok(())
}

/// Restores snapshot files to the instance. Takes the path to the snapshot file / directory.
fn restore_snapshot_files(
	snapshot_path: &Path,
	storage_type: StorageType,
	instance_dir: &Path,
) -> anyhow::Result<()> {
	match storage_type {
		StorageType::Archive => {
			let file = File::open(snapshot_path)?;
			let mut file = BufReader::new(file);
			let mut arc = ZipArchive::new(&mut file)?;
			arc.extract(instance_dir)
				.context("Failed to extract snapshot archive")?;
		}
		StorageType::Folder => {
			mcvm_core::io::files::copy_dir_contents(snapshot_path, instance_dir)
				.context("Failed to copy directory")?;
		}
	}

	Ok(())
}
