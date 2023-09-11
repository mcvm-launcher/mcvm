use mcvm_shared::output::{MCVMOutput, MessageContents, MessageLevel};
use mcvm_shared::versions::{VersionInfo, VersionPattern};

use crate::data::config::instance::{ClientWindowConfig, QuickPlay, WindowResolution};
use crate::data::instance::Instance;
use crate::data::user::{UserKind, UserManager};
use crate::io::files::paths::Paths;
use crate::io::java::classpath::Classpath;
use crate::net::game_files::assets::get_virtual_dir_path;
use crate::net::game_files::client_meta::args::ArgumentItem;
use crate::util::mojang::is_allowed;
use crate::util::{ARCH_STRING, OS_STRING};

/// Get the string for a placeholder token in an argument
macro_rules! placeholder {
	($name:expr) => {
		concat!("${", $name, "}")
	};
}

/// Replace placeholders in a string argument from the client meta
pub fn replace_arg_placeholders(
	instance: &Instance,
	arg: &str,
	paths: &Paths,
	users: &UserManager,
	classpath: &Classpath,
	version: &str,
	window: &ClientWindowConfig,
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
	out = out.replace(
		placeholder!("game_assets"),
		get_virtual_dir_path(paths).to_str()?,
	);
	out = out.replace(placeholder!("user_type"), "msa");
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
			if let Some(access_token) = &user.access_token {
				out = out.replace(placeholder!("auth_access_token"), access_token);
			}
			if let UserKind::Microsoft {
				xbox_uid: Some(xbox_uid),
			} = &user.kind
			{
				out = out.replace(placeholder!("auth_xuid"), xbox_uid);
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
				return Some("UnknownUser".into());
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

/// Process an argument for the client from the client meta
pub fn process_arg(
	instance: &Instance,
	arg: &ArgumentItem,
	paths: &Paths,
	users: &UserManager,
	classpath: &Classpath,
	version: &str,
	window: &ClientWindowConfig,
) -> Vec<String> {
	let mut out = Vec::new();
	match arg {
		ArgumentItem::Simple(arg) => {
			let arg = process_simple_arg(arg, instance, paths, users, classpath, version, window);
			if let Some(arg) = arg {
				out.push(arg);
			}
		}
		ArgumentItem::Conditional(arg) => {
			for rule in &arg.rules {
				let allowed = is_allowed(&rule.action.to_string());

				if let Some(os_name) = &rule.os.name {
					if allowed != (OS_STRING == os_name.to_string()) {
						return vec![];
					}
				}
				if let Some(os_arch) = &rule.os.arch {
					if allowed != (ARCH_STRING == os_arch.to_string()) {
						return vec![];
					}
				}

				if let Some(has_custom_resolution) = &rule.features.has_custom_resolution {
					if *has_custom_resolution && window.resolution.is_none() {
						return vec![];
					}
				}
				if let Some(is_demo_user) = &rule.features.is_demo_user {
					if *is_demo_user {
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

			for arg in arg.value.iter() {
				out.extend(process_simple_arg(
					arg, instance, paths, users, classpath, version, window,
				));
			}
		}
	};

	out
}

/// Process a simple string argument
pub fn process_simple_arg(
	arg: &str,
	instance: &Instance,
	paths: &Paths,
	users: &UserManager,
	classpath: &Classpath,
	version: &str,
	window: &ClientWindowConfig,
) -> Option<String> {
	replace_arg_placeholders(instance, arg, paths, users, classpath, version, window)
}

/// Create the game arguments for Quick Play
pub fn create_quick_play_args(
	quick_play: &QuickPlay,
	version_info: &VersionInfo,
	o: &mut impl MCVMOutput,
) -> Vec<String> {
	let mut out = Vec::new();

	match quick_play {
		QuickPlay::World { .. } | QuickPlay::Realm { .. } | QuickPlay::Server { .. } => {
			let after_23w14a = VersionPattern::After("23w14a".into()).matches_info(version_info);
			out.push("--quickPlayPath".into());
			out.push("quickPlay/log.json".into());
			match quick_play {
				QuickPlay::None => {}
				QuickPlay::World { world } => {
					if after_23w14a {
						out.push("--quickPlaySingleplayer".into());
						out.push(world.clone());
					} else {
						o.display(
							MessageContents::Warning(
								"World Quick Play has no effect before 23w14a (1.20)".into(),
							),
							MessageLevel::Important,
						);
					}
				}
				QuickPlay::Realm { realm } => {
					if after_23w14a {
						out.push("--quickPlayRealms".into());
						out.push(realm.clone());
					} else {
						o.display(
							MessageContents::Warning(
								"Realm Quick Play has no effect before 23w14a (1.20)".into(),
							),
							MessageLevel::Important,
						);
					}
				}
				QuickPlay::Server { server, port } => {
					if after_23w14a {
						out.push("--quickPlayMultiplayer".into());
						if let Some(port) = port {
							out.push(format!("{server}:{port}"));
						} else {
							out.push(server.clone());
						}
					} else {
						out.push("--server".into());
						out.push(server.clone());
						if let Some(port) = port {
							out.push("--port".into());
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

/// Fill the logging path argument with the correct path
pub fn fill_logging_path_arg(arg: String, version: &str, paths: &Paths) -> Option<String> {
	let path = crate::net::game_files::log_config::get_path(version, paths);
	Some(arg.replace(placeholder!("path"), path.to_str()?))
}