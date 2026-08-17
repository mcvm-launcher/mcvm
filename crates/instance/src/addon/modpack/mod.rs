use std::{
	fs::File,
	io::{Read, Seek},
	path::Path,
};

use anyhow::Context;
use zip::{ZipArchive, read::ZipFile};

/// CurseForge modpack format
pub mod cfpack;
/// Modrinth modpack format
pub mod mrpack;

/// Applies the given modpack override to the given path relative to the instance dir, preserving user-modified files
pub fn apply_zip_override<R: Read + Seek>(
	mut file: ZipFile<'_, R>,
	rel_target: &Path,
	instance_dir: &Path,
	old_pack: Option<&mut ZipArchive<R>>,
) -> anyhow::Result<()> {
	if file.is_dir() {
		return Ok(());
	}
	let Some(name) = file.enclosed_name() else {
		return Ok(());
	};

	let target_path = instance_dir.join(rel_target);

	if target_path.exists() {
		// If this was an override in the old pack that hasn't changed on the filesystem, we will let it update.
		let Some(old_pack) = old_pack else {
			return Ok(());
		};

		if !old_pack.file_names().any(|x| x == name.to_string_lossy()) {
			return Ok(());
		}

		let current_data =
			std::fs::read(&target_path).context("Failed to read existing override file")?;

		let mut old_file = old_pack
			.by_name(&name.to_string_lossy())
			.context("Failed to read old override file")?;
		let mut old_data = Vec::with_capacity(old_file.size() as usize);
		old_file
			.read_to_end(&mut old_data)
			.context("Failed to read old override file")?;

		if old_data != current_data {
			return Ok(());
		}
	}

	if let Some(parent) = target_path.parent() {
		let _ = std::fs::create_dir_all(parent);
	}
	let mut target_file = File::create(target_path)?;

	std::io::copy(&mut file, &mut target_file).context("Failed to copy file")?;

	Ok(())
}

/// Method for updating filesystem links
pub trait LinkMethod {
	/// Update a link, replacing it if it already exists
	fn link(&self, original: &Path, link: &Path) -> anyhow::Result<()>;
}

/// Default fs link method
pub struct DefaultLinkMethod;

impl LinkMethod for DefaultLinkMethod {
	fn link(&self, original: &Path, link: &Path) -> anyhow::Result<()> {
		if link.exists() {
			let _ = std::fs::remove_file(link);
		}
		std::fs::hard_link(original, link).context("Failed to hard link")
	}
}
