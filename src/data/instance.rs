use crate::lib::json;
use crate::lib::versions::MinecraftVersion;
use crate::net::helper;
use crate::net::helper::Download;
use crate::io::files::files;
use crate::Paths;

use color_print::cprintln;

pub enum InstKind {
	Client,
	Server
}

pub struct Instance {
	kind: InstKind,
	pub id: String,
	version: MinecraftVersion
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
	#[error("Failed to evaluate json file:\n\t{}", .0)]
	ParseError(#[from] json::JsonError),
	#[error("Error when downloading file:\n\t{}", .0)]
	Download(#[from] helper::DownloadError)
}

impl Instance {
	pub fn new(kind: InstKind, id: &str, version: &MinecraftVersion) -> Self {
		Self {
			kind,
			id: id.to_owned(),
			version: version.to_owned()
		}
	}

	// Create the data for the instance
	pub fn create(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		match &self.kind {
			InstKind::Client => {
				if force {
					cprintln!("<s>Rebuilding client <y>{}</y>", self.id);
				} else {
					cprintln!("<s>Updating client <y>{}</y>", self.id);
				}
				self.create_client(paths, verbose, force)?;
			},
			InstKind::Server => {
				if force {
					cprintln!("<s>Rebuilding server <b>{}</b>", self.id);
				} else {
					cprintln!("<s>Updating server <b>{}</b>", self.id);
				}
				self.create_server(paths, verbose, force)?
			}
		}
		Ok(())
	}

	fn create_client(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		let dir = paths.data.join("client").join(&self.id);
		files::create_leading_dirs(&dir).expect("Failed to create client directory");
		files::create_dir(&dir).expect("Failed to create client directory");
		Ok(())
	}

	fn create_server(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), CreateError> {
		Ok(())
	}
}
