use std::path::{Path, PathBuf};

use crate::{io::config::IO_CONFIG, util::OS_STRING};

/// IO configuration
pub mod config;

/// Tries to get the user's home dir
pub fn home_dir() -> anyhow::Result<PathBuf> {
	home_dir_from_os(OS_STRING)
}

/// Tries to get the user's home dir from the given OS string
pub fn home_dir_from_os(os: &str) -> anyhow::Result<PathBuf> {
	match os {
		"linux" => Ok(PathBuf::from(std::env::var("HOME")?)),
		"windows" => Ok(PathBuf::from(format!("{}/..", std::env::var("APPDATA")?))),
		"macos" => Ok(PathBuf::from(std::env::var("HOME")?)),
		_ => Ok(PathBuf::from("/")),
	}
}

/// Gets the configured IO link method
pub fn get_link_method() -> LinkMethod {
	let method = IO_CONFIG.get_string("link_method");
	let Some(method) = method else {
		// Hard links require admin privileges on Windows
		#[cfg(target_os = "windows")]
		return LinkMethod::Soft;
		#[cfg(not(target_os = "windows"))]
		return LinkMethod::Hard;
	};

	match method.as_str() {
		"hard" => LinkMethod::Hard,
		"soft" => LinkMethod::Soft,
		"copy" => LinkMethod::Copy,
		_ => LinkMethod::Hard,
	}
}

/// Different methods for files to be linked with
pub enum LinkMethod {
	/// Hardlink
	Hard,
	/// Symlink
	Soft,
	/// File is copied
	Copy,
}

/// Creates a new link if it does not exist
pub fn update_link(path: &Path, link: &Path) -> std::io::Result<()> {
	update_link_with_method(path, link, get_link_method())
}

/// Creates a new link if it does not exist with the given method
pub fn update_link_with_method(
	path: &Path,
	link: &Path,
	method: LinkMethod,
) -> std::io::Result<()> {
	if link.exists() {
		return Ok(());
	}

	match method {
		LinkMethod::Hard => std::fs::hard_link(path, link),
		LinkMethod::Soft =>
		{
			#[allow(deprecated)]
			std::fs::soft_link(path, link)
		}
		LinkMethod::Copy => {
			std::fs::copy(path, link)?;
			Ok(())
		}
	}
}

/// Gets the size of a directory recursively
pub fn dir_size(path: &Path) -> anyhow::Result<usize> {
	if !path.exists() {
		return Ok(0);
	}

	if path.is_file() {
		let meta = path.metadata()?;
		Ok(meta.len() as usize)
	} else {
		let mut sum = 0;
		let read = path.read_dir()?;
		for entry in read {
			let Ok(entry) = entry else {
				continue;
			};

			let Ok(size) = dir_size(&entry.path()) else {
				continue;
			};

			sum += size;
		}

		Ok(sum)
	}
}
