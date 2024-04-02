use std::path::{Path, PathBuf};

use anyhow::{ensure, Context};
use mcvm_shared::addon::{Addon, AddonKind};
use mcvm_shared::versions::{VersionInfo, VersionPattern};

use crate::data::addon::{self, AddonExt};
use crate::io::files::{self, paths::Paths, update_hardlink};

use super::{InstKind, Instance};

impl Instance {
	/// Creates an addon on the instance
	pub fn create_addon(
		&mut self,
		addon: &Addon,
		selected_worlds: &[String],
		paths: &Paths,
		version_info: &VersionInfo,
	) -> anyhow::Result<()> {
		self.ensure_dirs(paths)?;
		let game_dir = &self.dirs.get().game_dir;
		files::create_leading_dirs(game_dir)?;
		files::create_dir(game_dir)?;
		for path in self
			.get_linked_addon_paths(addon, selected_worlds, paths, version_info)
			.context("Failed to get linked directory")?
		{
			Self::link_addon(&path, addon, paths, &self.id)
				.with_context(|| format!("Failed to link addon {}", addon.id))?;
		}

		Ok(())
	}

	/// Get the paths on this instance to hardlink an addon to
	pub fn get_linked_addon_paths(
		&mut self,
		addon: &Addon,
		selected_worlds: &[String],
		paths: &Paths,
		version_info: &VersionInfo,
	) -> anyhow::Result<Vec<PathBuf>> {
		self.ensure_dirs(paths)?;
		let game_dir = &self.dirs.get().game_dir;
		Ok(match addon.kind {
			AddonKind::ResourcePack => {
				if let InstKind::Client { .. } = self.kind {
					// Resource packs are texture packs on older versions
					if VersionPattern::After("13w24a".into()).matches_info(version_info) {
						vec![game_dir.join("resourcepacks")]
					} else {
						vec![game_dir.join("texturepacks")]
					}
				} else {
					vec![]
				}
			}
			AddonKind::Mod => vec![game_dir.join("mods")],
			AddonKind::Plugin => {
				if let InstKind::Server { .. } = self.kind {
					vec![game_dir.join("plugins")]
				} else {
					vec![]
				}
			}
			AddonKind::Shader => {
				if let InstKind::Client { .. } = self.kind {
					vec![game_dir.join("shaderpacks")]
				} else {
					vec![]
				}
			}
			AddonKind::Datapack => {
				if let Some(datapack_folder) = &self.config.datapack_folder {
					vec![game_dir.join(datapack_folder)]
				} else {
					match &self.kind {
						InstKind::Client { .. } => {
							game_dir
								.join("saves")
								.read_dir()
								.context("Failed to read saves directory")?
								.filter_map(|world| {
									let world = world.ok()?;
									let path = world.path();
									// Filter worlds not in the list
									if !selected_worlds.is_empty() {
										let dir_name = path.file_name()?.to_string_lossy();
										if !selected_worlds.iter().any(|x| x == dir_name.as_ref()) {
											return None;
										}
									}
									Some(path.join("datapacks"))
								})
								.collect()
						}
						InstKind::Server { world_name, .. } => {
							let world_dir = world_name.as_deref().unwrap_or("world");
							vec![game_dir.join(world_dir).join("datapacks")]
						}
					}
				}
			}
		})
	}

	/// Hardlinks the addon from the path in addon storage to the correct in the instance,
	/// under the specified directory
	fn link_addon(
		dir: &Path,
		addon: &Addon,
		paths: &Paths,
		instance_id: &str,
	) -> anyhow::Result<()> {
		let link = dir.join(addon.file_name.clone());
		let addon_path = addon.get_path(paths, instance_id);
		files::create_leading_dirs(&link)?;
		// These checks are to make sure that we properly link the hardlink to the right location
		// We have to remove the current link since it doesnt let us update it in place
		ensure!(addon_path.exists(), "Addon path does not exist");
		if link.exists() {
			std::fs::remove_file(&link).context("Failed to remove instance addon file")?;
		}
		update_hardlink(&addon_path, &link).context("Failed to create hard link")?;
		Ok(())
	}

	/// Removes an addon file from this instance
	pub fn remove_addon_file(&self, path: &Path, paths: &Paths) -> anyhow::Result<()> {
		// We check if it is a stored addon path due to the old behavior to put that path in the lockfile.
		// Also some other sanity checks
		if path.exists() && !addon::is_stored_addon_path(path, paths) && !path.is_dir() {
			std::fs::remove_file(path).context("Failed to remove instance addon file")?;
		}

		Ok(())
	}
}
