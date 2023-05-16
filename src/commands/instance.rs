use anyhow::{anyhow, ensure, Context};
use clap::Subcommand;
use color_print::cprintln;
use mcvm::data::user::AuthState;

use mcvm::{data::instance::InstKind, util::print::HYPHEN_POINT};

use super::CmdData;

#[derive(Debug, Subcommand)]
pub enum InstanceSubcommand {
	#[command(about = "List all instances in all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
	},
	#[command(about = "Launch instances to play the game")]
	Launch {
		/// Whether to print the command that was generated when launching
		#[arg(short, long)]
		debug: bool,
		/// An optional user to choose when launching
		#[arg(short, long)]
		user: Option<String>,
		/// An optional Minecraft session token to override with
		#[arg(long)]
		token: Option<String>,
		/// The instance to launch
		instance: String,
	},
}

async fn list(data: &mut CmdData, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();
	for (id, instance) in config.instances.iter() {
		if raw {
			println!("{id}");
		} else {
			match instance.kind {
				InstKind::Client { .. } => cprintln!("{}<y!>{}", HYPHEN_POINT, id),
				InstKind::Server { .. } => cprintln!("{}<c!>{}", HYPHEN_POINT, id),
			}
		}
	}

	Ok(())
}

pub async fn launch(
	instance: &str,
	debug: bool,
	token: Option<String>,
	user: Option<String>,
	data: &mut CmdData,
) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config(true).await?;
	let paths = data.paths.get();
	let config = data.config.get_mut();

	if let Some(user) = user {
		ensure!(
			config.auth.users.contains_key(&user),
			"User '{user}' does not exist"
		);
		config.auth.state = AuthState::Authed(user);
	}

	let instance = config
		.instances
		.get_mut(instance)
		.ok_or(anyhow!("Unknown instance '{instance}'"))?;
	let (.., profile) = config
		.profiles
		.iter()
		.find(|(.., profile)| profile.instances.contains(&instance.id))
		.expect("Instance does not belong to any profiles");
	instance
		.launch(paths, &config.auth, debug, token, &profile.version)
		.await
		.context("Instance failed to launch")?;

	Ok(())
}

pub async fn run(command: InstanceSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match command {
		InstanceSubcommand::List { raw } => list(data, raw).await,
		InstanceSubcommand::Launch {
			debug,
			token,
			user,
			instance,
		} => launch(&instance, debug, token, user, data).await,
	}
}
