use std::path::{Path, PathBuf};

use anyhow::bail;

/// Installs the system java installation
pub fn install(major_version: &str) -> anyhow::Result<PathBuf> {
	let installation = get_system_java_installation(major_version);
	if let Some(installation) = installation {
		Ok(installation)
	} else {
		bail!("No valid system Java installation was found");
	}
}

/// Gets the optimal path to a system Java installation
fn get_system_java_installation(#[allow(unused_variables)] major_version: &str) -> Option<PathBuf> {
	#[cfg(target_os = "windows")]
	{
		if let Some(path) = scan_windows(major_version) {
			return Some(path);
		}
	}
	#[cfg(target_os = "linux")]
	{
		if let Some(path) = scan_linux(major_version) {
			return Some(path);
		}
	}

	None
}

/// Scan for Java on Windows
#[cfg(target_os = "windows")]
fn scan_windows(major_version: &str) -> Option<PathBuf> {
	// OpenJDK
	if let Some(path) = scan_dir(&PathBuf::from("C:/Program Files/Java"), major_version) {
		return Some(path);
	}

	None
}

/// Scan for Java on Linux
#[cfg(target_os = "linux")]
fn scan_linux(major_version: &str) -> Option<PathBuf> {
	// OpenJDK
	if let Some(path) = scan_dir(&PathBuf::from("/usr/lib/jvm"), major_version) {
		return Some(path);
	}

	None
}

/// Scan a directory for Java installations
fn scan_dir(dir: &Path, major_version: &str) -> Option<PathBuf> {
	if dir.exists() {
		let read = std::fs::read_dir(dir).ok()?;
		for path in read {
			let Ok(path) = path else { continue };
			if !path.path().is_dir() {
				continue;
			}
			let name = path.file_name().to_string_lossy().to_string();
			if !(name.starts_with("java-") || name.starts_with("jdk-")) {
				continue;
			}
			if !(name.contains(&format!("-{major_version}-"))
				|| name.contains(&format!("-{major_version}.")))
			{
				continue;
			}
			return Some(path.path());
		}
	}

	None
}
