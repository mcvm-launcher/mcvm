use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use mcvm_shared::util::utc_timestamp;
use rand::Rng;
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use zip::{ZipArchive, ZipWriter};

/// Name of the snapshot index file
pub const INDEX_NAME: &str = "index.json";

/// Settings for snapshots
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(default)]
pub struct Config {
	/// The max number of snapshots for an instance
	pub max_count: Option<u32>,
	/// The files and directories to include in the snapshot
	pub paths: Vec<String>,
	/// How the snapshot should be stored
	pub storage_type: StorageType,
}

/// Index for the snapshots of an instance
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Index {
	/// The list of available snapshots
	pub snapshots: Vec<Entry>,
}

impl Index {
	/// Gets the index path
	fn get_path(snapshot_directory: &Path) -> PathBuf {
		snapshot_directory.join(INDEX_NAME)
	}

	/// Open the index
	pub fn open(snapshot_directory: &Path) -> anyhow::Result<Self> {
		fs::create_dir_all(snapshot_directory)?;
		let path = Self::get_path(snapshot_directory);
		let contents = if path.exists() {
			let mut file = File::open(&path).context("Failed to open snapshot index")?;
			serde_json::from_reader(&mut file).context("Failed to parse JSON")?
		} else {
			Self::default()
		};

		Ok(contents)
	}

	/// Finish using the index
	pub fn finish(&self, snapshot_directory: &Path) -> anyhow::Result<()> {
		let path = Self::get_path(snapshot_directory);
		let mut file = File::create(path)?;
		serde_json::to_writer_pretty(&mut file, self)?;

		Ok(())
	}

	/// Checks if a snapshot with an ID exists already
	pub fn snapshot_exists(&self, snapshot_id: &str) -> bool {
		self.snapshots.iter().any(|x| x.id == snapshot_id)
	}

	/// Create a new snapshot
	pub fn create_snapshot(
		&mut self,
		kind: SnapshotKind,
		snapshot_id: String,
		config: &Config,
		instance_id: &str,
		instance_dir: &Path,
		snapshots_dir: &Path,
	) -> anyhow::Result<()> {
		// Remove the snapshot if it exists already
		if self.snapshot_exists(&snapshot_id) {
			self.remove_snapshot(&snapshot_id, instance_id, snapshots_dir)
				.context("Failed to remove existing snapshot with same ID")?;
		}

		let snapshot_dir = get_snapshot_directory(instance_id, snapshots_dir);

		let snapshot_path = get_snapshot_path(&snapshot_dir, &snapshot_id, config.storage_type);

		let mut readers = Vec::new();
		for path in &config.paths {
			let paths = get_instance_file_paths(path, instance_dir)
				.context("Failed to get recursive file paths")?;
			for path in paths {
				let file = File::open(instance_dir.join(&path))
					.with_context(|| format!("Failed to open snapshotted file with path {path}"))?;
				let file = BufReader::new(file);
				readers.push((path.clone(), file));
			}
		}
		write_snapshot_files(&snapshot_path, config, readers)?;

		let now = utc_timestamp()?;
		self.snapshots.push(Entry {
			id: snapshot_id,
			date: now,
			kind,
			storage_type: config.storage_type,
		});

		self.remove_old_snapshots(config, instance_id, snapshots_dir)?;

		Ok(())
	}

	/// Remove a snapshot
	pub fn remove_snapshot(
		&mut self,
		snapshot_id: &str,
		instance_id: &str,
		snapshot_dir: &Path,
	) -> anyhow::Result<()> {
		let index = self
			.snapshots
			.iter()
			.position(|x| x.id == snapshot_id)
			.ok_or(anyhow!("Snapshot with ID was not found"))?;
		let snapshot = &mut self.snapshots[index];

		let snapshot_dir = get_snapshot_directory(instance_id, snapshot_dir);
		let snapshot_path = get_snapshot_path(&snapshot_dir, snapshot_id, snapshot.storage_type);
		if snapshot_path.exists() {
			match snapshot.storage_type {
				StorageType::Archive => fs::remove_file(snapshot_path)?,
				StorageType::Folder => fs::remove_dir_all(snapshot_path)?,
			}
		}

		self.snapshots.remove(index);

		Ok(())
	}

	/// Remove old snapshots that are over the limit
	pub fn remove_old_snapshots(
		&mut self,
		config: &Config,
		instance_id: &str,
		snapshot_dir: &Path,
	) -> anyhow::Result<()> {
		if let Some(limit) = config.max_count {
			if self.snapshots.len() > (limit as usize) {
				let num_to_remove = self.snapshots.len() - (limit as usize);
				let to_remove: Vec<String> = self.snapshots[0..num_to_remove - 1]
					.iter()
					.map(|x| x.id.clone())
					.collect();
				for id in to_remove {
					self.remove_snapshot(&id, instance_id, snapshot_dir)
						.with_context(|| format!("Failed to remove old snapshot '{id}'"))?;
				}
			}
		}

		Ok(())
	}

	/// Restores a snapshot
	pub fn restore_snapshot(
		&self,
		snapshot_id: &str,
		instance_id: &str,
		instance_dir: &Path,
		snapshot_dir: &Path,
	) -> anyhow::Result<()> {
		let snapshot = self
			.snapshots
			.iter()
			.find(|x| x.id == snapshot_id)
			.ok_or(anyhow!("Snapshot with ID was not found"))?;

		let snapshot_dir = get_snapshot_directory(instance_id, snapshot_dir);
		let snapshot_path = get_snapshot_path(&snapshot_dir, snapshot_id, snapshot.storage_type);
		restore_snapshot_files(&snapshot_path, snapshot.storage_type, instance_dir)?;

		Ok(())
	}
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
pub fn get_snapshot_directory(instance_id: &str, base_dir: &Path) -> PathBuf {
	base_dir.join(instance_id)
}

/// Get the path to a specific snapshot
pub fn get_snapshot_path(
	snapshot_directory: &Path,
	snapshot_id: &str,
	storage_type: StorageType,
) -> PathBuf {
	let filename = match storage_type {
		StorageType::Archive => format!("{snapshot_id}.zip"),
		StorageType::Folder => snapshot_id.to_owned(),
	};

	snapshot_directory.join(filename)
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
	config: &Config,
	readers: Vec<(String, R)>,
) -> anyhow::Result<()> {
	match &config.storage_type {
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
