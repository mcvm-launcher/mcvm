/// Use of MCVM's configured system directories
pub mod paths;

use std::fs::{self, File};
use std::io::{BufWriter, Write};
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

/// Create all the directories leading up to a path
pub fn create_leading_dirs(path: &Path) -> std::io::Result<()> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}

	Ok(())
}

/// Create all the directories leading up to a path
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

/// Creates a new hardlink if it does not exist
pub async fn update_hardlink_async(path: &Path, link: &Path) -> std::io::Result<()> {
	if !link.exists() {
		tokio::fs::hard_link(path, link).await?;
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
#[cfg(target_family = "unix")]
pub fn dir_symlink(path: &Path, target: &Path) -> std::io::Result<()> {
	std::os::unix::fs::symlink(path, target)?;
	Ok(())
}


/// Copy the contents of a directory recursively to another directory.
/// Identical files will be overwritten
pub fn copy_dir_contents(src: &Path, dest: &Path) -> anyhow::Result<()> {
	ensure!(src.is_dir());
	ensure!(dest.is_dir());

	for file in fs::read_dir(src)? {
		let file = file?;
		let src_path = file.path();
		let rel = src_path.strip_prefix(src)?;
		let dest_path = dest.join(rel);

		let mut src_file = std::io::BufReader::new(std::fs::File::open(src_path)?);
		let mut dest_file = std::io::BufWriter::new(std::fs::File::create(dest_path)?);

		std::io::copy(&mut src_file, &mut dest_file)?;
	}

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

		let mut src_file = std::io::BufReader::new(std::fs::File::open(src_path)?);
		let mut dest_file = std::io::BufWriter::new(std::fs::File::create(dest_path)?);

		std::io::copy(&mut src_file, &mut dest_file)?;
	}

	Ok(())
}

/// Simple tweak of std::fs::write to use a BufWriter
pub fn write_buffered<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
	fn inner(path: &Path, contents: &[u8]) -> std::io::Result<()> {
		let mut file = BufWriter::new(File::create(path)?);
		file.write_all(contents)
	}
	inner(path.as_ref(), contents.as_ref())
}
