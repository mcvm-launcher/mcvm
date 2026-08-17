use std::path::{Path, PathBuf};

/// Get the path to a sha256 addon in storage
pub fn get_sha256_addon_path(addons_dir: &Path, hash: &str) -> PathBuf {
	addons_dir.join("sha256").join(hash)
}

/// Get the path to a generic addon in storage
pub fn get_generic_addon_path(
	addons_dir: &Path,
	addon_id: &str,
	addon_version: Option<String>,
) -> PathBuf {
	let base_dir = addons_dir.join("generic").join(addon_id);
	let addon_version = addon_version.unwrap_or("any".into());
	base_dir.join(addon_id).join(addon_version)
}
