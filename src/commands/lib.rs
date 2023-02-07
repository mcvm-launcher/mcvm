use crate::io::files::paths::{Paths, PathsError};
use crate::data::config::{Config, ConfigError};
use crate::data::instance::{CreateError, LaunchError};

use phf_macros::phf_map;

// Data passed to commands
pub struct CmdData {
	pub paths: Option<Paths>,
	pub config: Option<Config>
}

impl CmdData {
	pub fn new() -> Self {
		// let config_path = paths.project..join("mcvm.json");
		Self {
			paths: None,
			config: None
		}
	}

	pub fn ensure_paths(&mut self) -> Result<(), PathsError> {
		if let None = self.paths {
			self.paths = Some(Paths::new()?);
		}
		Ok(())
	}

	pub fn ensure_config(&mut self) -> Result<(), CmdError> {
		if let None = self.config {
			self.ensure_paths()?;
			if let Some(paths) = &self.paths {
				self.config = Some(Config::load(&paths.project.config_dir().join("mcvm.json"))?);
			}
		}
		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum CmdError {
	#[error("Failed to load config mcvm.json\n{}", .0)]
	Config(#[from] ConfigError),
	#[error("Failed to create paths:\n\t{}", .0)]
	Paths(#[from] PathsError),
	#[error("Failed to create profile:\n\t{}", .0)]
	ProfileCreate(#[from] CreateError),
	#[error("Failed to launch instance:\n\t{}", .0)]
	Launch(#[from] LaunchError),
	#[error("{}", .0)]
	Custom(String)
}

pub enum Command {
	Help,
	Profile,
	User,
	Launch
}

pub static COMMAND_MAP: phf::Map<&'static str, Command> = phf_map! {
	"help" => Command::Help,
	"profile" => Command::Profile,
	"user" => Command::User,
	"launch" => Command::Launch
};
