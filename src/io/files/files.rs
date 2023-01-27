use std::fs::create_dir;
use std::path::Path;

pub fn create_existing_dir(path: &Path) -> std::io::Result<()> {
	if path.exists() {
		Ok(())
	} else {
		create_dir(path)
	}
}
