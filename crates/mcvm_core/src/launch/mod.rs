/// Client-specific launch functionality
mod client;
/// Actual launching of the game process
mod process;
/// Server-specific launch functionality
mod server;

use std::collections::HashMap;
use std::path::Path;

use anyhow::Context;
use mcvm_shared::output::MCVMOutput;
use mcvm_shared::Side;

use self::client::create_quick_play_args;
use self::process::{launch_game_process, LaunchProcessParameters};
use crate::instance::InstanceKind;
use crate::io::files::paths::Paths;
use crate::io::java::args::{ArgsPreset, MemoryArg, MemoryNum};
use crate::io::java::classpath::Classpath;
use crate::io::java::install::{JavaInstallation, JavaInstallationKind};
use crate::net::game_files::client_meta::ClientMeta;
use crate::net::game_files::version_manifest::VersionManifestAndList;
use crate::user::UserManager;
use crate::util::versions::VersionName;

pub(crate) async fn launch(
	mut params: LaunchParameters<'_>,
	o: &mut impl MCVMOutput,
) -> anyhow::Result<InstanceHandle> {
	let command = params.java.get_jvm_path();
	// Get side-specific launch properties
	let props = match params.side.get_side() {
		Side::Client => self::client::get_launch_props(&mut params, o).await,
		Side::Server => self::server::get_launch_props(&params),
	}
	.context("Failed to generate side-specific launch properties")?;

	let user_access_token = params
		.users
		.get_chosen_user()
		.and_then(|x| x.get_access_token().cloned());

	let proc_params = LaunchProcessParameters {
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
}

/// Options for launching an instance
#[derive(Debug)]
pub struct LaunchConfiguration {
	/// Java kind
	pub java: JavaInstallationKind,
	/// JVM arguments
	pub jvm_args: Vec<String>,
	/// Game arguments
	pub game_args: Vec<String>,
	/// Minimum JVM memory
	pub min_mem: Option<MemoryNum>,
	/// Maximum JVM memory
	pub max_mem: Option<MemoryNum>,
	/// Java arguments preset
	pub preset: ArgsPreset,
	/// Environment variables
	pub env: HashMap<String, String>,
	/// Wrapper command
	pub wrapper: Option<WrapperCommand>,
	/// Quick Play options
	pub quick_play: QuickPlayType,
	/// Whether or not to use the Log4J configuration
	pub use_log4j_config: bool,
}

impl LaunchConfiguration {
	/// Create the args for the JVM when launching the game
	pub fn generate_jvm_args(&self) -> Vec<String> {
		let mut out = self.jvm_args.clone();

		if let Some(n) = &self.min_mem {
			out.push(MemoryArg::Min.to_string(n.clone()));
		}
		if let Some(n) = &self.max_mem {
			out.push(MemoryArg::Max.to_string(n.clone()));
		}

		let avg = match &self.min_mem {
			Some(min) => self
				.max_mem
				.as_ref()
				.map(|max| MemoryNum::avg(min.clone(), max.clone())),
			None => None,
		};
		out.extend(self.preset.generate_args(avg));

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

/// A wrapper command that can be used to
/// enclose the normal launch command in another
/// program.
#[derive(Debug, Clone)]
pub struct WrapperCommand {
	/// The command to run
	pub cmd: String,
	/// The command's arguments. These will be put after the argument
	/// for the normal launch command.
	pub args: Vec<String>,
}

/// Options for the Minecraft QuickPlay feature
#[derive(Debug, PartialEq, Default, Clone)]
pub enum QuickPlayType {
	/// QuickPlay a world
	World {
		/// The world to play
		world: String,
	},
	/// QuickPlay a server
	Server {
		/// The server address to join
		server: String,
		/// The port for the server to connect to.
		/// Uses the default port (25565) if not specified
		port: Option<u16>,
	},
	/// QuickPlay a realm
	Realm {
		/// The realm name to join
		realm: String,
	},
	/// Don't do any QuickPlay
	#[default]
	None,
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

	/// Gets the internal child process for the game
	pub fn get_process(&mut self) -> &mut std::process::Child {
		&mut self.process
	}
}
