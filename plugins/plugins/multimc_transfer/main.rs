use std::{
	collections::HashMap,
	fs::File,
	path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use nitro_core::io::{extract_zip_dir, files::copy_dir_contents, json_from_file};
use nitro_pkg::PkgRequest;
use nitro_plugin::{
	api::executable::ExecutablePlugin,
	hook::hooks::{CheckMigrationResult, ImportInstanceResult, MigrateInstancesResult},
};
use nitro_shared::{
	Side,
	loaders::Loader,
	minecraft::AddonKind,
	output::{MessageContents, NitroOutput},
	pkg::AddonOptionalHashes,
	versions::MinecraftVersionDeser,
};
use nitrolaunch::{
	config_crate::{
		instance::{InstanceConfig, make_valid_instance_id},
		package::PackageConfigDeser,
	},
	instance_crate::lock::{InstanceLockfile, LockfileAddon},
};
use serde::{Deserialize, Serialize};
use zip::ZipArchive;

fn main() -> anyhow::Result<()> {
	let mut plugin =
		ExecutablePlugin::from_manifest_file("multimc_transfer", include_str!("plugin.json"))?;

	plugin.import_instance(|_, arg| {
		let source_path = PathBuf::from(arg.source_path);
		let target_path = PathBuf::from(arg.result_path);

		let mut zip = ZipArchive::new(File::open(source_path).context("Failed to open instance")?)?;

		// Read the CFG file
		let mut cfg_file = zip
			.by_name("instance.cfg")
			.context("CFG file is missing in instance")?;
		let cfg =
			std::io::read_to_string(&mut cfg_file).context("Failed to read instance config")?;
		let cfg = read_instance_cfg(&cfg);
		std::mem::drop(cfg_file);

		let mut mmc_pack_file = zip
			.by_name("mmc-pack.json")
			.context("MMC Pack file is missing in instance")?;
		let mmc_pack: MMCPack = serde_json::from_reader(&mut mmc_pack_file)
			.context("Failed to deserialize instance MMC pack")?;

		std::mem::drop(mmc_pack_file);

		// Write the instance files

		let target_path = target_path.join(".minecraft");

		// Extract all the instance files
		extract_zip_dir(&mut zip, "minecraft", &target_path)
			.context("Failed to extract instance files")?;

		// Replace addons with packages
		let mut all_packages = Vec::new();

		for (addon_dir, addon_kind) in [
			("mods", AddonKind::Mod),
			("resourcepacks", AddonKind::ResourcePack),
			("texturepacks", AddonKind::ResourcePack),
			("shaderpacks", AddonKind::Shader),
			("shaders", AddonKind::Shader),
		] {
			all_packages.extend(addons_to_packages(
				&target_path.join(addon_dir),
				addon_kind,
			)?);
		}

		// Remove the existing package files
		for (_, path, _) in &all_packages {
			if path.exists() && path.is_file() {
				std::fs::remove_file(path).context("Failed to remove addon")?;
			}
		}

		let config = create_config(
			cfg,
			&mmc_pack,
			all_packages
				.into_iter()
				.map(|x| x.0.id.to_string())
				.collect(),
		)
		.context("Failed to create config")?;

		Ok(ImportInstanceResult {
			format: arg.format,
			config,
		})
	})?;

	plugin.check_migration(|mut ctx, arg| {
		if arg != "multimc" && arg != "prism" {
			return Ok(None);
		}

		let data_folder = data_folder(&arg)?;
		let instances_folder = data_folder.join("instances");
		let instances = get_available_instances(&instances_folder, ctx.get_output())?;

		Ok(instances.map(|instances| CheckMigrationResult { instances }))
	})?;

	plugin.migrate_instances(|mut ctx, arg| {
		// Get instances
		let data_folder = data_folder(&arg.format)?;
		let instances_folder = data_folder.join("instances");
		let names =
			get_available_instances(&instances_folder, ctx.get_output())?.unwrap_or_default();

		let nitro_data_dir = ctx.get_data_dir()?;
		let nitro_instances_dir = nitro_data_dir.join("instances");

		let mut instances = HashMap::new();

		for name in names {
			if let Some(requested_instances) = &arg.instances
				&& !requested_instances.is_empty()
				&& !requested_instances.contains(&name)
			{
				continue;
			}

			let path = instances_folder.join(&name);

			let mc_dir = path.join(".minecraft");
			let mc_dir = if mc_dir.exists() {
				mc_dir
			} else {
				path.join("minecraft")
			};

			let cfg_path = path.join("instance.cfg");

			let cfg =
				std::fs::read_to_string(cfg_path).context("Failed to read instance config")?;
			let cfg = read_instance_cfg(&cfg);

			let mmc_pack: MMCPack = json_from_file(path.join("mmc-pack.json"))
				.context("Failed to read instance MMC pack")?;

			let mut config =
				create_config(cfg, &mmc_pack, Vec::new()).context("Failed to create config")?;

			let id = make_valid_instance_id(config.name.as_ref().context("Instance has no name")?);

			// File migration
			if arg.link {
				config.dir = Some(mc_dir.to_string_lossy().to_string());
			} else {
				let target_dir = nitro_instances_dir.join(&id).join(".minecraft");
				std::fs::create_dir_all(&target_dir)?;
				copy_dir_contents(&mc_dir, &target_dir).context("Failed to copy instance files")?;
			}

			let id = if instances.contains_key(&id) {
				id + "2"
			} else {
				id
			};

			instances.insert(id.clone(), config);

			// Packages
			let mut inst_packages = Vec::new();

			for (addon_dir, addon_kind) in [
				("mods", AddonKind::Mod),
				("resourcepacks", AddonKind::ResourcePack),
				("texturepacks", AddonKind::ResourcePack),
				("shaderpacks", AddonKind::Shader),
				("shaders", AddonKind::Shader),
			] {
				inst_packages.extend(addons_to_packages(&mc_dir.join(addon_dir), addon_kind)?);
			}

			let mut lock = InstanceLockfile::open(&InstanceLockfile::get_path(
				Some(&mc_dir),
				&id,
				&nitro_data_dir.join("internal"),
			))
			.context("Failed to open lockfile")?;

			for (req, path, kind) in inst_packages {
				let addon_id = if req.repository == Some("modrinth".into()) {
					"addon"
				} else {
					"addon"
				};

				let addon = LockfileAddon {
					id: Some(addon_id.into()),
					package: Some(req.to_string_no_version_or_slug()),
					from_modpack: false,
					file_name: path.file_name().unwrap().to_string_lossy().to_string(),
					files: vec![path.to_string_lossy().to_string()],
					kind,
					hashes: AddonOptionalHashes::default(),
				};

				lock.update_package(&req, &[addon], None);
			}
		}

		Ok(MigrateInstancesResult {
			format: arg.format,
			instances,
		})
	})?;

	Ok(())
}

/// Creates the config for an instance from metadata
fn create_config(
	mut cfg: HashMap<&str, &str>,
	mmc_pack: &MMCPack,
	packages: Vec<String>,
) -> anyhow::Result<InstanceConfig> {
	let name = cfg.remove("name");

	let mut version = None;
	let mut loader = Loader::Vanilla;

	for component in &mmc_pack.components {
		if component.uid == "net.minecraft" {
			version = Some(component.version.clone());
		}
		if component.uid == "net.fabricmc.fabric-loader" {
			loader = Loader::Fabric;
		}
	}
	let version = version.context("No Minecraft version provided")?;

	Ok(InstanceConfig {
		name: name.map(|x| x.to_string()),
		side: Some(Side::Client),
		version: Some(MinecraftVersionDeser::Version(version.into())),
		loader: Some(loader.to_string().to_lowercase()),
		packages: packages
			.into_iter()
			.map(|x| PackageConfigDeser::Basic(x.into()))
			.collect(),
		..Default::default()
	})
}

/// Converts addons to packages in the given addon directory (resourcepacks, mods, etc.)
fn addons_to_packages(
	dir: &Path,
	addon_kind: AddonKind,
) -> anyhow::Result<Vec<(PkgRequest, PathBuf, AddonKind)>> {
	if !dir.exists() {
		return Ok(Vec::new());
	}

	// The .index dir in the addon dir will have a list of TOML files with info about each addon
	let index = dir.join(".index");
	if index.exists() {
		let mut out = Vec::new();
		for index_file in index.read_dir().context("Failed to read index directory")? {
			let index_file = index_file?;
			let contents = std::fs::read_to_string(index_file.path())
				.context("Failed to read index file contents")?;
			let toml = read_pw_toml(&contents);

			let global_section = toml.get("global").context("Global section missing")?;
			let filename = global_section.get("filename").context("Filename missing")?;

			if let Some(modrinth) = toml.get("update.modrinth") {
				let id = modrinth.get("mod-id").context("ID missing")?;

				out.push((
					PkgRequest::parse(
						format!("modrinth:{id}"),
						nitro_pkg::PkgRequestSource::UserRequire,
					),
					dir.join(filename),
					addon_kind,
				));
			}
		}

		Ok(out)
	} else {
		Ok(Vec::new())
	}
}

/// mmc-pack.json format
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MMCPack {
	components: Vec<MMCComponent>,
}

/// Single component in mmc-pack.json
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MMCComponent {
	cached_name: String,
	version: String,
	uid: String,
}

/// Reads the instance.cfg file to a set of fields
fn read_instance_cfg(contents: &str) -> HashMap<&str, &str> {
	let mut out = HashMap::new();

	for line in contents.lines() {
		let Some((key, value)) = line.split_once("=") else {
			continue;
		};

		out.insert(key, value);
	}

	out
}

/// Reads the .pw.toml file for an addon
fn read_pw_toml(contents: &str) -> HashMap<&str, HashMap<&str, &str>> {
	let mut sections: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
	let mut current_section = "global";

	for line in contents.lines() {
		if let Some((key, value)) = line.split_once(" = ") {
			// Remove quotes from strings
			let value = value
				.strip_prefix("'")
				.unwrap_or(value)
				.strip_suffix("'")
				.unwrap_or(value);

			sections
				.entry(current_section)
				.or_default()
				.insert(key, value);
		} else if line.starts_with("[") {
			current_section = &line[1..line.len() - 1]
		}
	}

	sections
}

/// Gets the list of available instance names, returning None specifically if the launcher instance folder does not exist
fn get_available_instances(
	instances_folder: &Path,
	o: &mut impl NitroOutput,
) -> anyhow::Result<Option<Vec<String>>> {
	if !instances_folder.exists() {
		return Ok(None);
	}

	let read = instances_folder
		.read_dir()
		.context("Failed to read instances")?;
	let mut out = Vec::new();
	for entry in read {
		let entry = entry?;

		if entry.file_type()?.is_file() {
			continue;
		}

		let name = entry.file_name().to_string_lossy().to_string();
		if name == "_LAUNCHER_TEMP" {
			continue;
		}

		let path = entry.path();

		let mc_dirs = [path.join(".minecraft"), path.join("minecraft")];
		if !mc_dirs.iter().any(|d| d.exists()) {
			o.debug(MessageContents::Simple(format!(
				"Skipping {name}, Minecraft dir missing"
			)));
			continue;
		}

		let cfg_path = path.join("instance.cfg");
		if !cfg_path.exists() {
			o.debug(MessageContents::Simple(format!(
				"Skipping {name}, Cfg file missing"
			)));
			continue;
		}

		out.push(name);
	}

	Ok(Some(out))
}

/// Gets the data folder depending on the format and operating system
fn data_folder(format: &str) -> anyhow::Result<PathBuf> {
	if format == "multimc" {
		#[cfg(target_os = "linux")]
		let data_folder = format!("{}/.local/share/multimc", std::env::var("HOME")?);
		#[cfg(target_os = "windows")]
		let data_folder = format!("{}/Roaming/MultiMC", std::env::var("APPDATA")?);
		#[cfg(target_os = "macos")]
		let data_folder = format!(
			"{}/Library/Application Support/MultiMC",
			std::env::var("HOME")?
		);

		Ok(PathBuf::from(data_folder))
	} else if format == "prism" {
		#[cfg(target_os = "linux")]
		let data_folder = format!("{}/.local/share/PrismLauncher", std::env::var("HOME")?);
		#[cfg(target_os = "windows")]
		let data_folder = format!("{}/Roaming/PrismLauncher", std::env::var("APPDATA")?);
		#[cfg(target_os = "macos")]
		let data_folder = format!(
			"{}/Library/Application Support/PrismLauncher",
			std::env::var("HOME")?
		);

		Ok(PathBuf::from(data_folder))
	} else {
		bail!("Unsupported format")
	}
}
