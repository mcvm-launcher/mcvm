use anyhow::{Context, bail};

use crate::data::instance::{Instance, InstKind};
use crate::data::user::{Auth, UserKind};
use crate::io::java::classpath::Classpath;
use crate::io::launch::launch;
use crate::util::json;
use crate::util::mojang::{is_allowed, ARCH_STRING, OS_STRING};
use crate::Paths;
use crate::{skip_fail, skip_none};

impl Instance {
	/// Launch a client
	pub fn launch_client(&mut self, paths: &Paths, auth: &Auth) -> anyhow::Result<()> {
		debug_assert!(self.kind == InstKind::Client);
		match &self.java {
			Some(java) => match &java.path {
				Some(java_path) => {
					let jre_path = java_path.join("bin/java");
					let client_dir = self.get_subdir(paths);
					let mut jvm_args = Vec::new();
					let mut game_args = Vec::new();

					if let Some(version_json) = &self.version_json {
						if let Some(classpath) = &self.classpath {
							let main_class = self
								.main_class
								.as_ref()
								.context("Main class is missing for client")?;
							if let Ok(args) = json::access_object(version_json, "arguments") {
								for arg in json::access_array(args, "jvm")? {
									for sub_arg in
										process_client_arg(self, arg, paths, auth, classpath)
									{
										jvm_args.push(sub_arg);
									}
								}

								for arg in json::access_array(args, "game")? {
									for sub_arg in
										process_client_arg(self, arg, paths, auth, classpath)
									{
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
										.join(&self.version)
										.join("natives")
										.to_str()
										.context("Failed to convert natives directory to a string")?
								));
								jvm_args.push(String::from("-cp"));
								jvm_args.push(classpath.get_str());

								for arg in args.split(' ') {
									game_args.push(skip_none!(process_string_arg(
										self, arg, paths, auth, classpath
									)));
								}
							}
							
							launch(
								paths,
								&self.id,
								&self.launch,
								false,
								&client_dir,
								jre_path.to_str().context("Failed to convert java path to a string")?,
								&jvm_args,
								Some(main_class),
								&game_args
							).context("Failed to run launch command")?;
						}
					}
					Ok(())
				}
				None => bail!("Java path is missing"),
			},
			None => bail!("Java installation missing"),
		}
	}
}

/// Replace tokens in a string argument from the version json
pub fn process_string_arg(
	instance: &Instance,
	arg: &str,
	paths: &Paths,
	auth: &Auth,
	classpath: &Classpath,
) -> Option<String> {
	let mut out = arg.replace("${launcher_name}", "mcvm");
	out = out.replace("${launcher_version}", "alpha");
	out = out.replace("${classpath}", &classpath.get_str());
	out = out.replace(
		"${natives_directory}",
		paths
			.internal
			.join("versions")
			.join(&instance.version)
			.join("natives")
			.to_str()?,
	);
	out = out.replace("${version_name}", &instance.version);
	out = out.replace("${version_type}", "mcvm");
	out = out.replace(
		"${game_directory}",
		instance
			.get_subdir(paths)
			.to_str()?,
	);
	out = out.replace(
		"${assets_root}",
		paths
			.assets
			.to_str()?,
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
			if out.contains("${auth_player_name}")
				|| out.contains("${auth_access_token}")
				|| out.contains("${auth_uuid}")
			{
				return Some(String::new());
			}
		}
		None => {
			if out.contains("${auth_player_name}")
				|| out.contains("${auth_access_token}")
				|| out.contains("${auth_uuid}")
			{
				return Some(String::new());
			}
		}
	}

	Some(out)
}

/// Process an argument for the client from the version json
pub fn process_client_arg(
	instance: &Instance,
	arg: &serde_json::Value,
	paths: &Paths,
	auth: &Auth,
	classpath: &Classpath,
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
			Some(value) => process_client_arg(instance, value, paths, auth, classpath),
			None => return vec![],
		};
	} else if let Some(contents) = arg.as_array() {
		for val in contents {
			out.push(
				process_client_arg(instance, val, paths, auth, classpath)
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
