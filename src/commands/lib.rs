use crate::io::files::Paths;
use crate::data::config::{Config, ConfigError};
use crate::data::instance::CreateError;

use phf_macros::phf_map;

// Data passed to commands
pub struct CmdData {
	pub paths: Paths,
	pub config: Config
}

impl CmdData {
	pub fn new() -> Self {
		let paths = Paths::new();
		let config_path = paths.config.join("mcvm.json");
		Self {
			paths,
			config: Config::new(&config_path)
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum CmdError {
	#[error("Failed to load config mcvm.json\n{}", .0)]
	Config(#[from] ConfigError),
	#[error("Failed to create profile:\n\t{}", .0)]
	ProfileCreate(#[from] CreateError),
	#[error("{}", .0)]
	Custom(String)
}

pub enum Command {
	Help,
	Profile,
	User
}

pub static COMMAND_MAP: phf::Map<&'static str, Command> = phf_map! {
	"help" => Command::Help,
	"profile" => Command::Profile,
	"user" => Command::User
};
