use std::path::PathBuf;

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
		// OpenJDK
		let dir = PathBuf::from("C:/Program Files/Java");
		if dir.exists() {
			let read = std::fs::read_dir(dir);
			if let Ok(read) = read {
				for path in read {
					let Ok(path) = path else { continue };
					if !path.path().is_dir() {
						continue;
					}
					let name = path.file_name().to_string_lossy().to_string();
					if !name.starts_with("jdk-") {
						continue;
					}
					if !name.contains(&format!("-{major_version}.")) {
						continue;
					}
					return Some(path.path());
				}
			}
		}
	}
	#[cfg(target_os = "linux")]
	{
		// OpenJDK
		let dir = PathBuf::from("/usr/lib/jvm");
		if dir.exists() {
			let read = std::fs::read_dir(dir);
			if let Ok(read) = read {
				for path in read {
					let Ok(path) = path else { continue };
					if !path.path().is_dir() {
						continue;
					}
					let name = path.file_name().to_string_lossy().to_string();
					if !name.starts_with("java-") {
						continue;
					}
					if !name.contains(&format!("-{major_version}-")) {
						continue;
					}
					return Some(path.path());
				}
			}
		}
	}
	None
}
