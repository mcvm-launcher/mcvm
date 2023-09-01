/// Use of MCVM's configured system directories
pub mod paths;

use std::fs;
use std::path::Path;

use anyhow::ensure;

/// Create a directory that may already exist without an error
pub fn create_dir(path: &Path) -> std::io::Result<()> {
	if path.exists() {
		Ok(())
	} else {
		fs::create_dir(path)
	}
}

/// Same as create_dir, but asynchronous
pub async fn create_dir_async(path: &Path) -> std::io::Result<()> {
	if path.exists() {
		Ok(())
	} else {
		tokio::fs::create_dir(path).await
	}
}

/// Create all the directories leading up to a path
pub fn create_leading_dirs(path: &Path) -> std::io::Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}

	Ok(())
}

/// Same as create_leading_dirs, but asynchronous
pub async fn create_leading_dirs_async(path: &Path) -> std::io::Result<()> {
	if let Some(parent) = path.parent() {
		tokio::fs::create_dir_all(parent).await?;
	}

	Ok(())
}

/// Creates a new hardlink if it does not exist
pub fn update_hardlink(path: &Path, link: &Path) -> std::io::Result<()> {
	if !link.exists() {
		fs::hard_link(path, link)?;
	}

	Ok(())
}

/// Cross platform - create a directory soft link
#[cfg(target_os = "windows")]
pub fn dir_symlink(path: &Path, target: &Path) -> std::io::Result<()> {
	std::os::windows::fs::symlink_dir(path, target)?;
	Ok(())
}

/// Cross platform - create a directory soft link
#[cfg(target_os = "linux")]
pub fn dir_symlink(path: &Path, target: &Path) -> std::io::Result<()> {
	std::os::unix::fs::symlink(path, target)?;
	Ok(())
}

/// Copy the contents of a directory recursively to another directory.
/// Identical files will be overwritten
pub async fn copy_dir_contents_async(src: &Path, dest: &Path) -> anyhow::Result<()> {
	ensure!(src.is_dir());
	ensure!(dest.is_dir());

	for file in fs::read_dir(src)? {
		let file = file?;
		let src_path = file.path();
		let rel = src_path.strip_prefix(src)?;
		let dest_path = dest.join(rel);

		let mut src_file = tokio::io::BufReader::new(tokio::fs::File::open(src_path).await?);
		let mut dest_file = tokio::io::BufWriter::new(tokio::fs::File::create(dest_path).await?);

		tokio::io::copy(&mut src_file, &mut dest_file).await?;
	}

	Ok(())
}
