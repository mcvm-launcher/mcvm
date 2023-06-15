use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};
use rand::Rng;
use serde::{Deserialize, Serialize};
use zip::{ZipArchive, ZipWriter};

use crate::util::utc_timestamp;

use super::files;
use super::files::paths::Paths;

pub static INDEX_NAME: &str = "index.json";

/// Type of a snapshot
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotKind {
	User,
}

/// Entry for a snapshot in the snapshot index
#[derive(Serialize, Deserialize)]
pub struct Entry {
	pub id: String,
	pub date: u64,
	pub kind: SnapshotKind,
	pub storage_type: StorageType,
}

/// Index for the snapshots of an instance
#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Index {
	pub snapshots: Vec<Entry>,
}

impl Index {
	/// Gets the index path
	fn get_path(snapshot_directory: &Path) -> PathBuf {
		snapshot_directory.join(INDEX_NAME)
	}

	/// Open the index
	pub fn open(snapshot_directory: &Path) -> anyhow::Result<Self> {
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

	/// Create a new snapshot
	pub fn create_snapshot(
		&mut self,
		kind: SnapshotKind,
		snapshot_id: String,
		config: &Config,
		instance_id: &str,
		instance_dir: &Path,
		paths: &Paths,
	) -> anyhow::Result<()> {
		let snapshot_dir = get_snapshot_directory(instance_id, paths);

		let snapshot_path = get_snapshot_path(&snapshot_dir, &snapshot_id, config.storage_type);
		let readers: Result<Vec<_>, anyhow::Error> = config
			.paths
			.iter()
			.map(|x| Ok((x.as_str(), File::open(instance_dir.join(x))?)))
			.collect();
		let readers = readers?;
		write_snapshot_files(&snapshot_path, config, readers)?;

		let now = utc_timestamp()?;
		self.snapshots.push(Entry {
			id: snapshot_id,
			date: now,
			kind,
			storage_type: config.storage_type.clone(),
		});

		self.remove_old_snapshots(config, instance_id, paths)?;

		Ok(())
	}

	/// Remove a snapshot
	pub fn remove_snapshot(
		&mut self,
		snapshot_id: &str,
		instance_id: &str,
		paths: &Paths,
	) -> anyhow::Result<()> {
		let index = self
			.snapshots
			.iter()
			.position(|x| x.id == snapshot_id)
			.ok_or(anyhow!("Snapshot with ID was not found"))?;
		let snapshot = &mut self.snapshots[index];

		let snapshot_dir = get_snapshot_directory(instance_id, paths);
		let snapshot_path = get_snapshot_path(&snapshot_dir, &snapshot_id, snapshot.storage_type);
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
		paths: &Paths,
	) -> anyhow::Result<()> {
		if let Some(limit) = config.max_count {
			if self.snapshots.len() > (limit as usize) {
				let num_to_remove = self.snapshots.len() - (limit as usize);
				let to_remove: Vec<String> = self.snapshots[0..num_to_remove - 1]
					.iter()
					.map(|x| x.id.clone())
					.collect();
				for id in to_remove {
					self.remove_snapshot(&id, instance_id, paths)
						.with_context(|| format!("Failed to remove old snapshot '{id}'"))?;
				}
			}
		}

		Ok(())
	}

	/// Restores a snapshot
	pub async fn restore_snapshot(
		&self,
		snapshot_id: &str,
		instance_id: &str,
		instance_dir: &Path,
		paths: &Paths,
	) -> anyhow::Result<()> {
		let snapshot = self
			.snapshots
			.iter()
			.find(|x| x.id == snapshot_id)
			.ok_or(anyhow!("Snapshot with ID was not found"))?;

		let snapshot_dir = get_snapshot_directory(instance_id, paths);
		let snapshot_path = get_snapshot_path(&snapshot_dir, &snapshot_id, snapshot.storage_type);
		restore_snapshot_files(&snapshot_path, snapshot.storage_type, instance_dir).await?;

		Ok(())
	}
}

/// Format for stored snapshots
#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum StorageType {
	Folder,
	#[default]
	Archive,
}

/// Settings for snapshots
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Config {
	pub max_count: Option<u32>,
	pub paths: Vec<String>,
	pub storage_type: StorageType,
}

/// Get the snapshot directory for an instance
pub fn get_snapshot_directory(instance_id: &str, paths: &Paths) -> PathBuf {
	paths.snapshots.join(instance_id)
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

/// Writes snapshot files to the stored format. Takes the path to the snapshot file / directory.
/// Readers are pairs of relative file paths and readers for files.
fn write_snapshot_files<'a, R: Read>(
	snapshot_path: &Path,
	config: &Config,
	readers: Vec<(&'a str, R)>,
) -> anyhow::Result<()> {
	match &config.storage_type {
		StorageType::Archive => {
			let file = File::create(snapshot_path)?;
			let mut file = BufWriter::new(file);
			let mut arc = ZipWriter::new(&mut file);
			let options = zip::write::FileOptions::default()
				.compression_method(zip::CompressionMethod::Stored);
			for (path, mut reader) in readers {
				arc.start_file(path, options)?;
				std::io::copy(&mut reader, &mut arc)?;
			}

			arc.finish()?;
		}
		StorageType::Folder => {
			for (path, mut reader) in readers {
				let dest = snapshot_path.join(path);
				files::create_leading_dirs(&dest)?;
				let file = File::create(dest)?;
				let mut file = BufWriter::new(file);
				std::io::copy(&mut reader, &mut file)?;
			}
		}
	};

	Ok(())
}

/// Restores snapshot files to the instance. Takes the path to the snapshot file / directory.
async fn restore_snapshot_files(
	snapshot_path: &Path,
	storage_type: StorageType,
	instance_dir: &Path,
) -> anyhow::Result<()> {
	match storage_type {
		StorageType::Archive => {
			let file = File::open(snapshot_path)?;
			let mut file = BufReader::new(file);
			let mut arc = ZipArchive::new(&mut file)?;
			for i in 0..arc.len() {
				let mut file = arc.by_index(i)?;
				let rel_path = PathBuf::from(
					file.enclosed_name()
						.context("Invalid compressed file path")?,
				);
				let out_path = instance_dir.join(rel_path);
				files::create_leading_dirs(&out_path)?;

				let mut out_file = File::create(&out_path)?;
				std::io::copy(&mut file, &mut out_file)
					.context("Failed to copy compressed file")?;
			}
		}
		StorageType::Folder => {
			files::copy_dir_contents_async(snapshot_path, instance_dir)
				.await
				.context("Failed to copy directory")?;
		}
	}

	Ok(())
}
