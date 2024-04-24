use std::collections::HashMap;

use crate::io::java::args::MemoryNum;
use crate::io::java::install::JavaInstallationKind;

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
	/// Environment variables
	pub env: HashMap<String, String>,
	/// Wrapper command
	pub wrappers: Vec<WrapperCommand>,
	/// Quick Play options
	pub quick_play: QuickPlayType,
	/// Whether or not to use the Log4J configuration
	pub use_log4j_config: bool,
}

impl LaunchConfiguration {
	/// Create a new LaunchConfiguration with default settings
	pub fn new() -> Self {
		Self {
			java: JavaInstallationKind::Auto,
			jvm_args: Vec::new(),
			game_args: Vec::new(),
			min_mem: None,
			max_mem: None,
			env: HashMap::new(),
			wrappers: Vec::new(),
			quick_play: QuickPlayType::None,
			use_log4j_config: false,
		}
	}

	/// Get a builder for the configuration
	pub fn builder() -> LaunchConfigBuilder {
		LaunchConfigBuilder::new()
	}
}

impl Default for LaunchConfiguration {
	fn default() -> Self {
		Self::new()
	}
}

/// Builder for the launch configuration
pub struct LaunchConfigBuilder {
	config: LaunchConfiguration,
}

impl LaunchConfigBuilder {
	/// Start a new ConfigBuilder with default configuration
	pub fn new() -> Self {
		Self {
			config: LaunchConfiguration::new(),
		}
	}

	/// Finish building and get the configuration
	pub fn build(self) -> LaunchConfiguration {
		self.config
	}

	/// Set the Java installation kind to use
	pub fn java(mut self, java: JavaInstallationKind) -> Self {
		self.config.java = java;
		self
	}

	/// Set additional JVM arguments to use
	pub fn jvm_args(mut self, jvm_args: Vec<String>) -> Self {
		self.config.jvm_args = jvm_args;
		self
	}

	/// Set additional game arguments to use
	pub fn game_args(mut self, game_args: Vec<String>) -> Self {
		self.config.game_args = game_args;
		self
	}

	/// Set the minimum memory for the JVM
	pub fn min_mem(mut self, min_mem: MemoryNum) -> Self {
		self.config.min_mem = Some(min_mem);
		self
	}

	/// Set the maximum memory for the JVM
	pub fn max_mem(mut self, max_mem: MemoryNum) -> Self {
		self.config.max_mem = Some(max_mem);
		self
	}

	/// Set environment variables for the command
	pub fn env(mut self, env: HashMap<String, String>) -> Self {
		self.config.env = env;
		self
	}

	/// Add a wrapper command that encloses the normal command
	pub fn wrapper(mut self, wrapper: WrapperCommand) -> Self {
		self.config.wrappers.push(wrapper);
		self
	}

	/// Set the type of Quick Play to use
	pub fn quick_play(mut self, quick_play: QuickPlayType) -> Self {
		self.config.quick_play = quick_play;
		self
	}

	/// Set whether to use the Log4J configuration
	pub fn use_log4j_config(mut self, use_log4j_config: bool) -> Self {
		self.config.use_log4j_config = use_log4j_config;
		self
	}
}

impl Default for LaunchConfigBuilder {
	fn default() -> Self {
		Self::new()
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
