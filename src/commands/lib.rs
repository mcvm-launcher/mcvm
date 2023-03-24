use crate::data::config::Config;
use crate::io::files::paths::Paths;

use phf_macros::phf_map;

// Data passed to commands
pub struct CmdData {
	pub paths: Option<Paths>,
	pub config: Option<Config>,
}

impl CmdData {
	pub fn new() -> Self {
		Self {
			paths: None,
			config: None,
		}
	}

	pub fn ensure_paths(&mut self) -> anyhow::Result<()> {
		if self.paths.is_none() {
			self.paths = Some(Paths::new()?);
		}
		Ok(())
	}

	pub fn ensure_config(&mut self) -> anyhow::Result<()> {
		if self.config.is_none() {
			self.ensure_paths()?;
			if let Some(paths) = &self.paths {
				self.config = Some(Config::load(&paths.project.config_dir().join("mcvm.json"))?);
			}
		}
		Ok(())
	}
}

pub enum Command {
	Help,
	Profile,
	User,
	Launch,
	Version,
	Files,
	Package,
}

pub static COMMAND_MAP: phf::Map<&'static str, Command> = phf_map! {
	"help" => Command::Help,
	"profile" => Command::Profile,
	"user" => Command::User,
	"launch" => Command::Launch,
	"version" => Command::Version,
	"files" => Command::Files,
	"package" => Command::Package
};
