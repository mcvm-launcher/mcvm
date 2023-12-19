use std::path::PathBuf;

use super::files::paths::Paths;

/// Get the path to the options file
pub fn get_path(paths: &Paths) -> PathBuf {
	paths.project.config_dir().join("options.json")
}
