pub mod paths;

use std::fs;
use std::path::Path;

// Create a directory that may already exist
pub fn create_dir(path: &Path) -> std::io::Result<()> {
	if path.exists() {
		Ok(())
	} else {
		fs::create_dir(path)
	}
}

// Create all the directories leading up to a path
pub fn create_leading_dirs(path: &Path) -> std::io::Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	Ok(())
}

// Cross platform - create a directory soft link
#[cfg(target_os = "windows")]
pub fn dir_symlink(path: &Path, target: &Path) -> std::io::Result<()> {
	std::os::windows::fs::symlink_dir(path, target)?;
	Ok(())
}

#[cfg(target_os = "linux")]
pub fn dir_symlink(path: &Path, target: &Path) -> std::io::Result<()> {
	std::os::unix::fs::symlink(path, target)?;
	Ok(())
}
