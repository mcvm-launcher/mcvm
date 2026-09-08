use std::{
	path::{Path, PathBuf},
	process::Command,
};

use crate::{io::home_dir_from_os, pkg::ArcPkgReq, util::open_link};
use serde::{Deserialize, Serialize};

/// A file that the user needs to manually download
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManualFile {
	/// The name of the file, used for matching it in the scan directory and matching it up with the addon
	pub filename: String,
	/// The page where the user can download the file
	pub url: String,
	/// Package associated with this file, if any
	pub req: Option<ArcPkgReq>,
}

/// Gets the manual scan directory for the given OS
pub fn get_scan_dir_from_os(os: &str) -> PathBuf {
	home_dir_from_os(os)
		.unwrap_or_else(|_| PathBuf::from("/"))
		.join("Downloads")
}

/// Gets the filenames of the files that have been manually downloaded by the user
pub fn scan(dir: &Path, files: &[ManualFile]) -> Vec<String> {
	let mut found_files = Vec::new();

	for file in files {
		let path = dir.join(&file.filename);
		if path.exists() {
			found_files.push(file.filename.clone());
		}
	}

	found_files
}

/// Opens all the manual files in the user's browser
pub fn open_all(files: &[ManualFile]) {
	if files.len() > 1 {
		if try_new_window(files).is_ok() {
			return;
		}
	}

	for file in files {
		let _ = open_link(&file.url);
	}
}

/// Tries to open all the links in a new window
fn try_new_window(files: &[ManualFile]) -> anyhow::Result<()> {
	if files.is_empty() {
		return Ok(());
	}

	let to_try: &[&str] = if cfg!(target_os = "windows") {
		&["firefox.exe", "chrome.exe", "msedge.exe"]
	} else {
		&["firefox", "google-chrome"]
	};

	for exe in to_try {
		let first_file = &files[0];
		let command = Command::new(exe)
			.arg("--new-window")
			.arg(&first_file.url)
			.args(files.iter().skip(1).map(|x| &x.url))
			.spawn();
		if command.is_ok() {
			return Ok(());
		}
	}

	Err(anyhow::anyhow!("Failed to open a new window"))
}
