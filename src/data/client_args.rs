use super::user::{Auth, UserKind};
use crate::util::mojang::{is_allowed, OS_STRING, ARCH_STRING};
use crate::{skip_fail, skip_none};
use crate::Paths;
use crate::util::json;
use super::instance::Instance;

pub fn process_string_arg(
	instance: &Instance,
	arg: &str,
	paths: &Paths,
	auth: &Auth,
	classpath: &str
) -> Option<String> {
	let mut out = arg.replace("${launcher_name}", "mcvm");
	out = out.replace("${launcher_version}", "alpha");
	out = out.replace("${classpath}", classpath);
	out = out.replace(
		"${natives_directory}",
		paths.internal.join("versions").join(&instance.version).join("natives").to_str()
			.expect("Failed to convert natives directory to a string")
	);
	out = out.replace("${version_name}", &instance.version);
	out = out.replace("${version_type}", "mcvm");
	out = out.replace(
		"${game_directory}",
		instance.get_subdir(paths).to_str()
			.expect("Failed to convert client directory to a string")
	);
	out = out.replace(
		"${assets_root}",
		paths.assets.to_str().expect("Failed to convert assets directory to a string")
	);
	out = out.replace("${assets_index_name}", &instance.version);
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
			if
				out.contains("${auth_player_name}")
				|| out.contains("${auth_access_token}")
				|| out.contains("${auth_uuid}")
			{
				return Some(String::new());
			}
		},
		None => if
			out.contains("${auth_player_name}")
			|| out.contains("${auth_access_token}")
			|| out.contains("${auth_uuid}")
		{
			return Some(String::new());
		}
	}

	Some(out)
}

// Process an argument for the client from the version json
pub fn process_client_arg(
	instance: &Instance,
	arg: &serde_json::Value,
	paths: &Paths,
	auth: &Auth,
	classpath: &str
) -> Vec<String> {
	let mut out = Vec::new();
	if let Some(contents) = arg.as_str() {
		let processed = process_string_arg(instance, contents, paths, auth, classpath);
		if let Some(processed_arg) = processed {
			out.push(processed_arg);
		}
	} else if let Some(contents) = arg.as_object() {
		let rules = match json::access_array(contents, "rules") {
			Ok(rules) => rules,
			Err(..) => return vec![]
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
						None => false
					};
					if fail {
						return vec![];
					}
				}
			}
		}
		match arg.get("value") {
			Some(value) => process_client_arg(instance, value, paths, auth, classpath),
			None => return vec![]
		};
	} else if let Some(contents) = arg.as_array() {
		for val in contents {
			out.push(
				process_client_arg(instance, val, paths, auth, classpath).get(0).expect("Expected an argument").to_string()
			);
		}
	} else {
		return vec![];
	}

	out
}
