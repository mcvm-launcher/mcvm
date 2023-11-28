use std::collections::HashSet;
use std::path::{Path, PathBuf};

use mcvm_shared::util::print::PrintOptions;

/// Manager for when we are updating profile files.
/// It will keep track of files we have already downloaded, manage task requirements, etc
#[derive(Debug)]
pub struct UpdateManager {
	/// Options for printing / output
	pub print: PrintOptions,
	/// Whether to force file updates
	pub force: bool,
	/// Whether we will prioritize local files instead of remote ones
	pub allow_offline: bool,
	/// File paths that are added when they have been updated by other functions
	files: HashSet<PathBuf>,
}

impl UpdateManager {
	/// Create a new UpdateManager
	pub fn new(print: PrintOptions, force: bool, allow_offline: bool) -> Self {
		Self {
			print,
			force,
			allow_offline,
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
		if self.force {
			!self.files.contains(file) || !file.exists()
		} else {
			!file.exists()
		}
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
