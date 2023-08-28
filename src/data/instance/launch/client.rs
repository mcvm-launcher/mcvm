use std::collections::HashMap;

use anyhow::Context;
use color_print::cprintln;

use crate::data::config::instance::QuickPlay;
use crate::data::config::instance::{ClientWindowConfig, WindowResolution};
use crate::data::instance::launch::LaunchProcessProperties;
use crate::data::instance::{InstKind, Instance};
use crate::data::user::{UserKind, UserManager};
use crate::io::files::paths::Paths;
use crate::io::java::classpath::Classpath;
use crate::util::json;
use crate::util::{
	mojang::is_allowed,
	{ARCH_STRING, OS_STRING},
};
use crate::{skip_fail, skip_none};
use mcvm_shared::versions::VersionInfo;
use mcvm_shared::versions::VersionPattern;

pub use args::create_quick_play_args;

impl Instance {
	/// Launch a client
	pub fn launch_client(
		&mut self,
		paths: &Paths,
		users: &UserManager,
		debug: bool,
		token: Option<String>,
		version_info: &VersionInfo,
	) -> anyhow::Result<()> {
		debug_assert!(matches!(self.kind, InstKind::Client { .. }));
		let java_path = self.java.get().path.get();
		let jre_path = java_path.join("bin/java");
		let client_dir = self.get_subdir(paths);
		let mut jvm_args = Vec::new();
		let mut game_args = Vec::new();
		let client_json = self.client_json.get();
		if let Some(classpath) = &self.classpath {
			let main_class = self
				.main_class
				.as_ref()
				.expect("Main class for client should exist");
			if let InstKind::Client { options: _, window } = &self.kind {
				if let Ok(args) = json::access_object(client_json, "arguments") {
					for arg in json::access_array(args, "jvm")? {
						for sub_arg in args::process_arg(
							self,
							arg,
							paths,
							users,
							classpath,
							&version_info.version,
							window,
							&token,
						) {
							jvm_args.push(sub_arg);
						}
					}

					for arg in json::access_array(args, "game")? {
						for sub_arg in args::process_arg(
							self,
							arg,
							paths,
							users,
							classpath,
							&version_info.version,
							window,
							&token,
						) {
							game_args.push(sub_arg);
						}
					}
				} else {
					// Behavior for versions prior to 1.12.2
					let args = json::access_str(client_json, "minecraftArguments")?;

					jvm_args.push(format!(
						"-Djava.library.path={}",
						paths
							.internal
							.join("versions")
							.join(&version_info.version)
							.join("natives")
							.to_str()
							.context("Failed to convert natives directory to a string")?
					));
					jvm_args.push(String::from("-cp"));
					jvm_args.push(classpath.get_str());

					for arg in args.split(' ') {
						game_args.push(skip_none!(args::replace_arg_placeholders(
							self,
							arg,
							paths,
							users,
							classpath,
							&version_info.version,
							window,
							&token
						)));
					}
				}
			}

			let mut env_vars = HashMap::new();
			// Compatability env var for old versions on Linux to prevent graphical issues
			#[cfg(target_os = "linux")]
			{
				if VersionPattern::from("1.8.9-").matches_info(version_info) {
					env_vars.insert("__GL_THREADED_OPTIMIZATIONS".to_string(), "0".to_string());
				}
			}

			let launch_properties = LaunchProcessProperties {
				cwd: &client_dir,
				command: jre_path
					.to_str()
					.context("Failed to convert java path to a string")?,
				jvm_args: &jvm_args,
				main_class: Some(main_class),
				game_args: &game_args,
				additional_env_vars: &env_vars,
			};

			self.launch_game_process(launch_properties, debug, version_info, paths)
				.context("Failed to launch game process")?;
		}

		Ok(())
	}
}

mod args {
	use super::*;

	/// Get the string for a placeholder token in an argument
	macro_rules! placeholder {
		($name:expr) => {
			concat!("${", $name, "}")
		};
	}

	/// Replace placeholders in a string argument from the client JSON
	pub fn replace_arg_placeholders(
		instance: &Instance,
		arg: &str,
		paths: &Paths,
		users: &UserManager,
		classpath: &Classpath,
		version: &str,
		window: &ClientWindowConfig,
		token: &Option<String>,
	) -> Option<String> {
		let mut out = arg.replace(placeholder!("launcher_name"), "mcvm");
		out = out.replace(placeholder!("launcher_version"), "alpha");
		out = out.replace(placeholder!("classpath"), &classpath.get_str());
		out = out.replace(
			placeholder!("natives_directory"),
			paths
				.internal
				.join("versions")
				.join(version)
				.join("natives")
				.to_str()?,
		);
		out = out.replace(placeholder!("version_name"), version);
		out = out.replace(placeholder!("version_type"), "mcvm");
		out = out.replace(
			placeholder!("game_directory"),
			instance.get_subdir(paths).to_str()?,
		);
		out = out.replace(placeholder!("assets_root"), paths.assets.to_str()?);
		out = out.replace(placeholder!("assets_index_name"), version);
		out = out.replace(placeholder!("user_type"), "mojang");
		out = out.replace(placeholder!("clientid"), "mcvm");
		// Apparently this is used for Twitch on older versions
		out = out.replace(placeholder!("user_properties"), "\"\"");

		// Window resolution
		if let Some(WindowResolution { width, height }) = window.resolution {
			out = out.replace(placeholder!("resolution_width"), &width.to_string());
			out = out.replace(placeholder!("resolution_height"), &height.to_string());
		}

		// User
		match users.get_user() {
			Some(user) => {
				out = out.replace(placeholder!("auth_player_name"), &user.name);
				if let Some(uuid) = &user.uuid {
					out = out.replace(placeholder!("auth_uuid"), uuid);
				}
				if let Some(token) = token {
					out = out.replace(placeholder!("auth_access_token"), token);
				} else if let Some(access_token) = &user.access_token {
					out = out.replace(placeholder!("auth_access_token"), access_token);
				}
				if let Some(xbox_uid) = &user.xbox_uid {
					out = out.replace(placeholder!("auth_xuid"), &xbox_uid);
				}

				// Blank any args we don't replace since the game will complain if we don't
				if out.contains(placeholder!("auth_player_name"))
					|| out.contains(placeholder!("auth_access_token"))
					|| out.contains(placeholder!("auth_uuid"))
				{
					return Some(String::new());
				}
			}
			None => {
				if out.contains(placeholder!("auth_player_name")) {
					return Some(String::from("UnknownUser"));
				}
				if out.contains(placeholder!("auth_access_token"))
					|| out.contains(placeholder!("auth_uuid"))
				{
					return Some(String::new());
				}
			}
		}

		Some(out)
	}

	/// Process an argument for the client from the client JSON
	pub fn process_arg(
		instance: &Instance,
		arg: &serde_json::Value,
		paths: &Paths,
		users: &UserManager,
		classpath: &Classpath,
		version: &str,
		window: &ClientWindowConfig,
		token: &Option<String>,
	) -> Vec<String> {
		let mut out = Vec::new();
		if let Some(contents) = arg.as_str() {
			let processed = replace_arg_placeholders(
				instance, contents, paths, users, classpath, version, window, token,
			);
			if let Some(processed_arg) = processed {
				out.push(processed_arg);
			}
		} else if let Some(contents) = arg.as_object() {
			let rules = match json::access_array(contents, "rules") {
				Ok(rules) => rules,
				Err(..) => return vec![],
			};
			for rule_val in rules.iter() {
				let rule = skip_none!(rule_val.as_object());
				let allowed = is_allowed(skip_fail!(json::access_str(rule, "action")));
				if let Some(os_val) = rule.get("os") {
					let os = skip_none!(os_val.as_object());
					if let Some(os_name) = os.get("name") {
						if allowed != (OS_STRING == skip_none!(os_name.as_str())) {
							return vec![];
						}
					}
					if let Some(os_arch) = os.get("arch") {
						if allowed != (ARCH_STRING == skip_none!(os_arch.as_str())) {
							return vec![];
						}
					}
				}
				if let Some(features_val) = rule.get("features") {
					let features = skip_none!(features_val.as_object());
					if features.get("has_custom_resolution").is_some()
						&& window.resolution.is_none()
					{
						return vec![];
					}
					if features.get("is_demo_user").is_some() {
						let fail = match users.get_user() {
							Some(user) => matches!(user.kind, UserKind::Demo),
							None => false,
						};
						if fail {
							return vec![];
						}
					}
				}
			}
			match arg.get("value") {
				Some(value) => process_arg(
					instance, value, paths, users, classpath, version, window, token,
				),
				None => return vec![],
			};
		} else if let Some(contents) = arg.as_array() {
			for val in contents {
				out.push(
					process_arg(
						instance, val, paths, users, classpath, version, window, token,
					)
					.get(0)
					.expect("Expected an argument")
					.to_string(),
				);
			}
		} else {
			return vec![];
		}

		out
	}

	/// Create the game arguments for Quick Play
	pub fn create_quick_play_args(
		quick_play: &QuickPlay,
		version_info: &VersionInfo,
	) -> Vec<String> {
		let mut out = Vec::new();

		match quick_play {
			QuickPlay::World { .. } | QuickPlay::Realm { .. } | QuickPlay::Server { .. } => {
				let after_23w14a =
					VersionPattern::After(String::from("23w14a")).matches_info(version_info);
				out.push(String::from("--quickPlayPath"));
				out.push(String::from("quickPlay/log.json"));
				match quick_play {
					QuickPlay::None => {}
					QuickPlay::World { world } => {
						if after_23w14a {
							out.push(String::from("--quickPlaySingleplayer"));
							out.push(world.clone());
						} else {
							cprintln!(
								"<y>Warning: World Quick Play has no effect before 23w14a (1.20)"
							);
						}
					}
					QuickPlay::Realm { realm } => {
						if after_23w14a {
							out.push(String::from("--quickPlayRealms"));
							out.push(realm.clone());
						} else {
							cprintln!(
								"<y>Warning: Realm Quick Play has no effect before 23w14a (1.20)"
							);
						}
					}
					QuickPlay::Server { server, port } => {
						if after_23w14a {
							out.push(String::from("--quickPlayMultiplayer"));
							if let Some(port) = port {
								out.push(format!("{server}:{port}"));
							} else {
								out.push(server.clone());
							}
						} else {
							out.push(String::from("--server"));
							out.push(server.clone());
							if let Some(port) = port {
								out.push(String::from("--port"));
								out.push(port.to_string());
							}
						}
					}
				}
			}
			_ => {}
		}

		out
	}
}
