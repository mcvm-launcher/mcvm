use anyhow::Context;
use color_print::cprintln;

use crate::data::config::instance::QuickPlay;
use crate::data::instance::{InstKind, Instance};
use crate::data::user::{Auth, UserKind};
use crate::io::java::classpath::Classpath;
use crate::io::launch::{launch, LaunchArgument};
use crate::util::json;
use crate::util::versions::VersionPattern;
use crate::util::{
	mojang::is_allowed,
	{ARCH_STRING, OS_STRING}
};
use crate::Paths;
use crate::{skip_fail, skip_none};

pub use args::create_quick_play_args;

impl Instance {
	/// Launch a client
	pub fn launch_client(
		&mut self,
		paths: &Paths,
		auth: &Auth,
		debug: bool,
		version: &str,
		version_list: &[String],
	) -> anyhow::Result<()> {
		debug_assert!(matches!(self.kind, InstKind::Client { .. }));
		let java_path = self.java.get().path.get();
		let jre_path = java_path.join("bin/java");
		let client_dir = self.get_subdir(paths);
		let mut jvm_args = Vec::new();
		let mut game_args = Vec::new();
		let version_json = self.version_json.get();
		if let Some(classpath) = &self.classpath {
			let main_class = self.main_class.as_ref().expect("Main class for client should exist");
			if let Ok(args) = json::access_object(version_json, "arguments") {
				for arg in json::access_array(args, "jvm")? {
					for sub_arg in args::process_arg(
						self, arg, paths, auth, classpath, version,
					) {
						jvm_args.push(sub_arg);
					}
				}

				for arg in json::access_array(args, "game")? {
					for sub_arg in args::process_arg(
						self, arg, paths, auth, classpath, version,
					) {
						game_args.push(sub_arg);
					}
				}
			} else {
				// Behavior for versions prior to 1.12.2
				let args = json::access_str(version_json, "minecraftArguments")?;

				jvm_args.push(format!(
					"-Djava.library.path={}",
					paths
						.internal
						.join("versions")
						.join(version)
						.join("natives")
						.to_str()
						.context(
							"Failed to convert natives directory to a string"
						)?
				));
				jvm_args.push(String::from("-cp"));
				jvm_args.push(classpath.get_str());

				for arg in args.split(' ') {
					game_args.push(skip_none!(args::replace_arg_tokens(
						self, arg, paths, auth, classpath, version
					)));
				}
			}

			let launch_argument = LaunchArgument {
				instance_name: &self.id,
				side: self.kind.to_side(),
				options: &self.launch,
				debug,
				version,
				version_list,
				cwd: &client_dir,
				command: jre_path
					.to_str()
					.context("Failed to convert java path to a string")?,
				jvm_args: &jvm_args,
				main_class: Some(main_class),
				game_args: &game_args,
			};

			launch(paths, &launch_argument)
				.context("Failed to run launch command")?;
		}

		Ok(())
	}
}

mod args {
	use super::*;
		
	/// Replace tokens in a string argument from the version json
	pub fn replace_arg_tokens(
		instance: &Instance,
		arg: &str,
		paths: &Paths,
		auth: &Auth,
		classpath: &Classpath,
		version: &str,
	) -> Option<String> {
		let mut out = arg.replace("${launcher_name}", "mcvm");
		out = out.replace("${launcher_version}", "alpha");
		out = out.replace("${classpath}", &classpath.get_str());
		out = out.replace(
			"${natives_directory}",
			paths
				.internal
				.join("versions")
				.join(version)
				.join("natives")
				.to_str()?,
		);
		out = out.replace("${version_name}", version);
		out = out.replace("${version_type}", "mcvm");
		out = out.replace("${game_directory}", instance.get_subdir(paths).to_str()?);
		out = out.replace("${assets_root}", paths.assets.to_str()?);
		out = out.replace("${assets_index_name}", version);
		out = out.replace("${user_type}", "mojang");
		out = out.replace("${clientid}", "mcvm");
		out = out.replace("${auth_xuid}", "mcvm");
		// Apparently this is used for Twitch on older versions
		out = out.replace("${user_properties}", "\"\"");

		// User
		match auth.get_user() {
			Some(user) => {
				out = out.replace("${auth_player_name}", &user.name);
				if let Some(uuid) = &user.uuid {
					out = out.replace("${auth_uuid}", uuid);
				}
				if let Some(token) = &user.access_token {
					out = out.replace("${auth_access_token}", token);
				}
				if out.contains("${auth_player_name}")
					|| out.contains("${auth_access_token}")
					|| out.contains("${auth_uuid}")
				{
					return Some(String::new());
				}
			}
			None => {
				if out.contains("${auth_player_name}") {
					return Some(String::from("UnknownUser"));
				}
				if out.contains("${auth_access_token}")
					|| out.contains("${auth_uuid}")
				{
					return Some(String::new());
				}
			}
		}

		Some(out)
	}

	/// Process an argument for the client from the version json
	pub fn process_arg(
		instance: &Instance,
		arg: &serde_json::Value,
		paths: &Paths,
		auth: &Auth,
		classpath: &Classpath,
		version: &str,
	) -> Vec<String> {
		let mut out = Vec::new();
		if let Some(contents) = arg.as_str() {
			let processed = replace_arg_tokens(instance, contents, paths, auth, classpath, version);
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
					if features.get("has_custom_resolution").is_some() {
						return vec![];
					}
					if features.get("is_demo_user").is_some() {
						let fail = match auth.get_user() {
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
				Some(value) => process_arg(instance, value, paths, auth, classpath, version),
				None => return vec![],
			};
		} else if let Some(contents) = arg.as_array() {
			for val in contents {
				out.push(
					process_arg(instance, val, paths, auth, classpath, version)
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
		version: &str,
		version_list: &[String]
	) -> Vec<String> {
		let mut out = Vec::new();

		match quick_play {
			QuickPlay::World { .. } | QuickPlay::Realm { .. } | QuickPlay::Server { .. } => {
				let after_23w14a = VersionPattern::After(String::from("23w14a"))
					.matches_single(version, version_list);
				out.push(String::from("--quickPlayPath"));
				out.push(String::from("quickPlay/log.json"));
				match quick_play {
					QuickPlay::None => {}
					QuickPlay::World { world } => {
						if after_23w14a {
							out.push(String::from("--quickPlaySingleplayer"));
							out.push(world.clone());
						} else {
							cprintln!("<y>Warning: World Quick Play has no effect before 23w14a (1.20)");
						}
					}
					QuickPlay::Realm { realm } => {
						if after_23w14a {
							out.push(String::from("--quickPlayRealms"));
							out.push(realm.clone());
						} else {
							cprintln!("<y>Warning: Realm Quick Play has no effect before 23w14a (1.20)");
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
