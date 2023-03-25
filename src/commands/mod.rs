mod files;
pub mod help;
mod launch;
pub mod lib;
pub mod package;
mod profile;
mod user;
mod version;

use anyhow::anyhow;
use lib::{CmdData, Command, COMMAND_MAP};

impl Command {
	pub async fn run(
		&self,
		argc: usize,
		argv: &[String],
		data: &mut CmdData,
	) -> anyhow::Result<()> {
		match self {
			Self::Help => help::run(argc, argv, data),
			Self::Profile => profile::run(argc, argv, data).await,
			Self::User => user::run(argc, argv, data),
			Self::Launch => launch::run(argc, argv, data).await,
			Self::Version => version::run(argc, argv, data),
			Self::Files => files::run(argc, argv, data),
			Self::Package => package::run(argc, argv, data).await,
		}
	}

	pub fn help(&self) {
		match self {
			Self::Help => help::help(),
			Self::Profile => profile::help(),
			Self::User => user::help(),
			Self::Launch => launch::help(),
			Self::Version => version::help(),
			Self::Files => files::help(),
			Self::Package => package::help(),
		}
	}
}

pub async fn run_command(
	command: &str,
	argc: usize,
	argv: &[String],
	data: &mut CmdData
) -> anyhow::Result<()> {
	let command = COMMAND_MAP.get(command)
		.ok_or(anyhow!("{} is not a valid command\nRun <b>mcvm help</b> for a list of commands.", command))?;
	command.run(argc, argv, data).await?;

	Ok(())
}
