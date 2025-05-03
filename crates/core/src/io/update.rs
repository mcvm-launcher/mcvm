use std::collections::HashSet;
use std::path::{Path, PathBuf};

use mcvm_shared::UpdateDepth;

/// Manager for when we are updating profile files.
/// It will keep track of files we have already downloaded, manage task requirements, etc
#[derive(Debug)]
pub struct UpdateManager {
	/// The depth to perform updates at.
	pub(crate) update_depth: UpdateDepth,
	/// File paths that are added when they have been updated by other functions
	files: HashSet<PathBuf>,
}

impl UpdateManager {
	/// Create a new UpdateManager
	pub fn new(depth: UpdateDepth) -> Self {
		Self {
			update_depth: depth,
			files: HashSet::new(),
		}
	}

	/// Add a single tracked file to the manager
	pub fn add_file(&mut self, file: PathBuf) {
		self.files.insert(file);
	}

	/// Add tracked files to the manager
	pub fn add_files(&mut self, files: HashSet<PathBuf>) {
		self.files.extend(files);
	}

	/// Adds an UpdateMethodResult to the UpdateManager
	pub fn add_result(&mut self, result: UpdateMethodResult) {
		self.add_files(result.files_updated);
	}

	/// Whether a file needs to be updated
	pub fn should_update_file(&self, file: &Path) -> bool {
		if self.update_depth == UpdateDepth::Force {
			!self.files.contains(file) || !file.exists()
		} else {
			!file.exists()
		}
	}

	/// Gets the update depth of the manager
	pub fn get_depth(&self) -> UpdateDepth {
		self.update_depth
	}
}

/// Struct returned by updating functions, with data like changed files
#[derive(Default)]
pub struct UpdateMethodResult {
	/// The files that this function has updated
	pub files_updated: HashSet<PathBuf>,
}

impl UpdateMethodResult {
	/// Create a new UpdateMethodResult
	pub fn new() -> Self {
		Self::default()
	}

	/// Merges this result with another one
	pub fn merge(&mut self, other: Self) {
		self.files_updated.extend(other.files_updated);
	}
}
