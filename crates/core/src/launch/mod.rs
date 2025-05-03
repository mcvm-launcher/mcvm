/// Client-specific launch functionality
mod client;
/// Configuration for launch settings
mod configuration;
/// Actual launching of the game process
mod process;
/// Server-specific launch functionality
mod server;

use std::path::Path;

use anyhow::{bail, Context};
use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::{translate, Side};

use self::client::create_quick_play_args;
use self::process::{launch_game_process, LaunchGameProcessParameters};
use crate::config::BrandingProperties;
use crate::instance::InstanceKind;
use crate::io::files::paths::Paths;
use crate::io::java::args::MemoryArg;
use crate::io::java::classpath::Classpath;
use crate::io::java::install::JavaInstallation;
use crate::net::game_files::client_meta::ClientMeta;
use crate::net::game_files::version_manifest::VersionManifestAndList;
use crate::user::auth::check_game_ownership;
use crate::user::UserManager;
use crate::util::versions::VersionName;

pub use self::configuration::{
	LaunchConfigBuilder, LaunchConfiguration, QuickPlayType, WrapperCommand,
};

pub use self::process::launch_process;
pub use self::process::{LaunchProcessParameters, LaunchProcessProperties};

pub(crate) async fn launch(
	params: LaunchParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<InstanceHandle> {
	let command = params.java.get_jvm_path();

	// Make sure we are authenticated
	if let InstanceKind::Client { .. } = &params.side {
		o.display(
			MessageContents::StartProcess(translate!(o, StartAuthenticating)),
			MessageLevel::Important,
		);

		params
			.users
			.authenticate(params.paths, params.req_client, o)
			.await
			.context("Failed to ensure authentication")?;

		// Ensure game ownership in case we are using an alternative auth system
		let owns_game =
			check_game_ownership(params.paths).context("Failed to check for game ownership")?;

		if !owns_game {
			bail!("Could not prove game ownership. If using an alternative auth system, like from a plugin, you must login with a Microsoft account that owns Minecraft first.");
		}
	}

	// Get side-specific launch properties
	let props = match params.side.get_side() {
		Side::Client => self::client::get_launch_props(&params).await,
		Side::Server => self::server::get_launch_props(&params),
	}
	.context("Failed to generate side-specific launch properties")?;

	let user_access_token = params
		.users
		.get_chosen_user()
		.and_then(|x| x.get_access_token());

	let proc_params = LaunchGameProcessParameters {
		command: command.as_os_str(),
		cwd: params.launch_dir,
		main_class: Some(params.main_class),
		props,
		launch_config: params.launch_config,
		version: params.version,
		version_list: &params.version_manifest.list,
		side: params.side,
		user_access_token,
		censor_secrets: params.censor_secrets,
	};

	let child = launch_game_process(proc_params, o).context("Failed to launch game process")?;

	let handle = InstanceHandle::new(child);
	Ok(handle)
}

/// Container struct for parameters for launching an instance
pub(crate) struct LaunchParameters<'a> {
	pub version: &'a VersionName,
	pub version_manifest: &'a VersionManifestAndList,
	pub side: &'a InstanceKind,
	pub launch_dir: &'a Path,
	pub java: &'a JavaInstallation,
	pub classpath: &'a Classpath,
	pub main_class: &'a str,
	pub launch_config: &'a LaunchConfiguration,
	pub paths: &'a Paths,
	pub req_client: &'a reqwest::Client,
	pub client_meta: &'a ClientMeta,
	pub users: &'a mut UserManager,
	pub censor_secrets: bool,
	pub branding: &'a BrandingProperties,
}

impl LaunchConfiguration {
	/// Create the args for the JVM when launching the game
	pub fn generate_jvm_args(&self) -> Vec<String> {
		let mut out = self.jvm_args.clone();

		if let Some(n) = &self.min_mem {
			out.push(MemoryArg::Min.to_string(n));
		}
		if let Some(n) = &self.max_mem {
			out.push(MemoryArg::Max.to_string(n));
		}

		out
	}

	/// Create the args for the game when launching
	pub fn generate_game_args(
		&self,
		version: &str,
		version_list: &[String],
		side: Side,
		o: &mut impl MCVMOutput,
	) -> Vec<String> {
		let mut out = self.game_args.clone();

		out.extend(self.generate_additional_game_args(version, version_list, side, o));

		out
	}

	/// Create additional args for the game when launching
	pub fn generate_additional_game_args(
		&self,
		version: &str,
		version_list: &[String],
		side: Side,
		o: &mut impl MCVMOutput,
	) -> Vec<String> {
		let mut out = Vec::new();

		if let Side::Client = side {
			out.extend(create_quick_play_args(
				&self.quick_play,
				version,
				version_list,
				o,
			));
		}

		out
	}
}

/// Handle for an instance after launching it. You must make sure to use
/// .wait() so that the child process is awaited.
#[derive(Debug)]
pub struct InstanceHandle {
	/// The child process for the launched instance
	process: std::process::Child,
}

impl InstanceHandle {
	/// Construct a new InstanceHandle
	fn new(process: std::process::Child) -> Self {
		Self { process }
	}

	/// Waits for the process to complete
	pub fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
		self.process.wait()
	}

	/// Kills the process early
	pub fn kill(&mut self) -> std::io::Result<()> {
		self.process.kill()
	}

	/// Gets the internal child process for the game, consuming the
	/// InstanceHandle
	pub fn get_process(self) -> std::process::Child {
		self.process
	}

	/// Gets the PID of the instance process
	pub fn get_pid(&self) -> u32 {
		self.process.id()
	}
}
