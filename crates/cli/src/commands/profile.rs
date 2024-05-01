use super::CmdData;
use itertools::Itertools;
use mcvm::core::util::versions::{MinecraftLatestVersion, MinecraftVersionDeser};
use mcvm::data::config::builder::ProfileBuilder;
use mcvm::data::config::modifications::{apply_modifications_and_write, ConfigModification};
use mcvm::data::profile::update::update_profiles;
use mcvm::shared::id::ProfileID;
use mcvm::shared::modifications::ClientType;
use mcvm::shared::modifications::{Modloader, ServerType};

use anyhow::{bail, Context};
use clap::Subcommand;
use color_print::{cprint, cprintln};
use mcvm::shared::Side;
use reqwest::Client;

use crate::output::{icons_enabled, HYPHEN_POINT, INSTANCE, LOADER, PACKAGE, VERSION};

#[derive(Debug, Subcommand)]
pub enum ProfileSubcommand {
	#[command(about = "Print useful information about a profile")]
	Info { profile: String },
	#[command(about = "List all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
	},
	#[command(
		about = "Update a profile",
		long_about = "Update the game files, extensions, packages, and addons of a profile."
	)]
	Update {
		/// Whether to force update files that have already been downloaded
		#[arg(short, long)]
		force: bool,
		/// Whether to update all profiles
		#[arg(short, long)]
		all: bool,
		/// Whether to skip updating packages
		#[arg(short = 'P', long)]
		skip_packages: bool,
		/// The profiles to update
		profiles: Vec<String>,
	},
	#[command(about = "Add new profiles to your config")]
	Add {},
	#[command(about = "Launch the proxy on a profile")]
	ProxyLaunch {
		/// The profile which has the proxy to launch
		profile: String,
	},
}

pub async fn run(subcommand: ProfileSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		ProfileSubcommand::Info { profile } => info(data, &profile).await,
		ProfileSubcommand::List { raw } => list(data, raw).await,
		ProfileSubcommand::Update {
			force,
			all,
			profiles,
			skip_packages,
		} => update(data, &profiles, force, all, skip_packages).await,
		ProfileSubcommand::Add {} => add(data).await,
		ProfileSubcommand::ProxyLaunch { profile } => proxy_launch(data, profile).await,
	}
}

async fn info(data: &mut CmdData, id: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	fn print_indent() {
		print!("   ");
	}

	if let Some(profile) = config.profiles.get(id) {
		cprintln!("<s><g>Profile <b>{}", id);
		print_indent();
		if icons_enabled() {
			print!("{} ", VERSION);
		}
		cprintln!("<s>Version:</s> <g>{}", profile.version);

		if profile.modifications.common_modloader() {
			print_indent();
			if icons_enabled() {
				print!("{} ", LOADER);
			}
			cprintln!(
				"<s>Modloader:</s> <g>{}",
				profile.modifications.get_modloader(Side::Client)
			);
		} else {
			print_indent();
			if icons_enabled() {
				print!("{} ", LOADER);
			}
			cprintln!("<s>Client:</s> <g>{}", profile.modifications.client_type);
			print_indent();
			if icons_enabled() {
				print!("{} ", LOADER);
			}
			cprintln!("<s>Server:</s> <g>{}", profile.modifications.server_type);
		}

		print_indent();
		if icons_enabled() {
			print!("{} ", INSTANCE);
		}
		cprintln!("<s>Instances:");
		for (inst_id, instance) in profile.instances.iter() {
			print_indent();
			cprint!("{}", HYPHEN_POINT);
			match instance.get_side() {
				Side::Client => cprint!("<y!>Client {}", inst_id),
				Side::Server => cprint!("<c!>Server {}", inst_id),
			}
			cprintln!();
		}
		print_indent();
		if icons_enabled() {
			print!("{} ", PACKAGE);
		}
		cprintln!("<s>Packages:");
		for pkg in profile.packages.iter_global() {
			print_indent();
			cprint!("{}", HYPHEN_POINT);
			cprint!("<b!>{}<g!>", pkg);
			cprintln!();
		}
	} else {
		bail!("Unknown profile '{id}'");
	}

	Ok(())
}

async fn list(data: &mut CmdData, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get();

	if !raw {
		cprintln!("<s>Profiles:");
	}
	for (id, profile) in config.profiles.iter().sorted_by_key(|x| x.0) {
		if raw {
			println!("{id}");
		} else {
			cprintln!("<s><g>   {}", id);
			for (inst_id, instance) in profile.instances.iter() {
				match instance.get_side() {
					Side::Client => cprintln!("   {}<y!>{}", HYPHEN_POINT, inst_id),
					Side::Server => cprintln!("   {}<c!>{}", HYPHEN_POINT, inst_id),
				}
			}
		}
	}

	Ok(())
}

async fn update(
	data: &mut CmdData,
	ids: &[String],
	force: bool,
	all: bool,
	skip_packages: bool,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let ids: Vec<ProfileID> = if all {
		config.profiles.keys().cloned().collect()
	} else {
		ids.iter().cloned().map(ProfileID::from).collect()
	};

	update_profiles(
		&data.paths,
		config,
		&ids,
		force,
		!skip_packages,
		&mut data.output,
	)
	.await?;

	Ok(())
}

async fn add(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let mut config = data.get_raw_config()?;

	// Build the profile
	let id = inquire::Text::new("What is the ID for the profile?").prompt()?;
	let version = inquire::Text::new("What Minecraft version should the profile be?").prompt()?;
	let version = match version.as_str() {
		"latest" => MinecraftVersionDeser::Latest(MinecraftLatestVersion::Release),
		"latest_snapshot" => MinecraftVersionDeser::Latest(MinecraftLatestVersion::Snapshot),
		other => MinecraftVersionDeser::Version(other.into()),
	};
	let mut profile = ProfileBuilder::new(id.into(), version);

	let options = vec![Modloader::Vanilla, Modloader::Fabric, Modloader::Quilt];
	let modloader =
		inquire::Select::new("What modloader should the profile use?", options).prompt()?;
	profile.modloader(modloader);

	let options = vec![
		ClientType::None,
		ClientType::Vanilla,
		ClientType::Fabric,
		ClientType::Quilt,
	];
	let client_type = inquire::Select::new(
		"What client type should the profile use? Select 'None' to inherit from the modloader",
		options,
	)
	.prompt()?;
	profile.client_type(client_type);

	let options = vec![
		ServerType::None,
		ServerType::Vanilla,
		ServerType::Fabric,
		ServerType::Quilt,
		ServerType::Paper,
		ServerType::Sponge,
		ServerType::Folia,
	];
	let server_type = inquire::Select::new(
		"What server type should the profile use? Select 'None' to inherit from the modloader",
		options,
	)
	.prompt()?;
	profile.server_type(server_type);

	let (profile_id, profile) = profile.build_inner();

	apply_modifications_and_write(
		&mut config,
		vec![ConfigModification::AddProfile(profile_id, profile)],
		&data.paths,
	)
	.context("Failed to write modified config")?;

	cprintln!("<g>Profile added.");

	Ok(())
}

async fn proxy_launch(data: &mut CmdData, profile: String) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let profile = ProfileID::from(profile);
	let profile = config
		.profiles
		.get_mut(&profile)
		.context(format!("Profile '{profile}' does not exist"))?;

	let client = Client::new();

	let child = profile
		.launch_proxy(&client, &data.paths, &config.plugins, &mut data.output)
		.await
		.context("Failed to launch proxy")?;

	if let Some(mut child) = child {
		child.wait().context("Failed to wait for child process")?;
	}

	Ok(())
}
