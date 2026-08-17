use std::{
	ops::DerefMut,
	path::{Path, PathBuf},
	sync::Arc,
};

use anyhow::{Context, bail};
use nitro_config::instance::{InstanceConfig, make_valid_instance_id};
use nitro_core::io::files::create_leading_dirs_async;
use nitro_instance::lock::{LockfileAddon, LockfileModpack};
use nitro_net::download;
use nitro_plugin::hook::hooks::{AddModpackFormats, InstallModpack, InstallModpackArg};
use nitro_shared::{
	Side, UpdateDepth,
	lang::Language,
	loaders::Loader,
	minecraft::AddonKind,
	output::{MessageContents, NitroOutput},
	pkg::{ArcPkgReq, PackageStability, merge_package_lists},
	versions::VersionInfo,
};
use reqwest::Client;

use crate::{
	addon::AddonExt,
	instance::{Instance, transfer::load_formats, update::InstanceUpdateContext},
	io::paths::Paths,
	pkg::{
		eval::{EvalConstants, EvalInput, EvalParameters, Routine},
		reg::PkgRegistry,
	},
	plugin::PluginManager,
};

impl Instance {
	/// Updates a package modpack on an instance
	pub async fn update_modpack<'a, O: NitroOutput>(
		&mut self,
		modpack: &ArcPkgReq,
		depth: UpdateDepth,
		version_info: &VersionInfo,
		ctx: &mut InstanceUpdateContext<'a, O>,
	) -> anyhow::Result<ModpackInstallResult> {
		let mut inst_lock = self.get_lockfile(ctx.paths)?;

		if self.dir.is_none() {
			return Ok(ModpackInstallResult::default());
		}

		// Modpack already installed
		let lock_modpack = inst_lock.get_modpack();
		if depth <= UpdateDepth::Shallow
			&& let Some(modpack) = lock_modpack
		{
			return Ok(ModpackInstallResult {
				supplied_packages: modpack.packages.clone(),
			});
		}

		let mut section = ctx.output.get_section();
		section.display(MessageContents::Header("Updating modpack".into()));

		let mut process = section.get_process();
		process.display(MessageContents::StartProcess("Downloading modpack".into()));

		let constants = EvalConstants {
			version: Some(version_info.version.clone()),
			loader: self.loader.clone(),
			version_list: version_info.versions.clone(),
			language: ctx.prefs.language,
			default_stability: self.config.package_stability.unwrap_or_default(),
			suppress: Vec::new(),
		};
		let params = EvalParameters::new(self.side());

		let input = EvalInput {
			constants: Arc::new(constants),
			params,
		};

		let download_result = download_modpack_package(
			modpack,
			input,
			&self.id,
			ctx.packages,
			ctx.plugins,
			ctx.client,
			ctx.paths,
			process.deref_mut(),
		)
		.await
		.context("Failed to download modpack")?;

		let Some(download_result) = download_result else {
			return Ok(ModpackInstallResult::default());
		};

		process.display(MessageContents::Success("Modpack downloaded".into()));
		process.finish();

		let formats = ctx
			.plugins
			.call_hook(AddModpackFormats, &(), ctx.paths, section.deref_mut())
			.await?
			.flatten_all_results_with_ids(section.deref_mut())
			.await?;

		let Some((plugin_id, format)) = formats.iter().find(|x| x.1.id == download_result.format)
		else {
			bail!(
				"Modpack format {} is not supported. Try installing a plugin for it.",
				download_result.format
			);
		};

		let modpack_path_str = download_result.modpack_path.to_string_lossy().to_string();

		// Don't supply the old modpack path if it is the same as the new one
		let old_path = if let Some(lock_modpack) = lock_modpack {
			if lock_modpack.path == modpack_path_str {
				None
			} else {
				if !Path::new(&lock_modpack.path).exists() {
					section.display(MessageContents::Warning(
						"Old modpack not available. Update may not work properly".into(),
					));
				}

				Some(lock_modpack.path.clone())
			}
		} else {
			None
		};

		let version_manifest = ctx
			.core
			.get_version_manifest(None, depth, section.deref_mut())
			.await
			.context("Failed to get Minecraft version manifest")?;

		let arg = InstallModpackArg {
			format: format.id.clone(),
			path: modpack_path_str.clone(),
			old_path,
			target_path: self.dir().unwrap().to_string_lossy().to_string(),
			side: self.side(),
			minecraft_versions: version_manifest.list.clone(),
		};

		let result = ctx
			.plugins
			.call_hook_on_plugin(
				InstallModpack,
				plugin_id,
				&arg,
				ctx.paths,
				section.deref_mut(),
			)
			.await?;

		let result = result.context("Modpack install was not handled by plugin")?;
		let result = result
			.result(section.deref_mut())
			.await
			.context("Failed to install modpack")?;

		let addons: Vec<_> = result
			.addons
			.into_iter()
			.map(|x| LockfileAddon {
				id: None,
				package: None,
				from_modpack: true,
				file_name: x.file_name,
				files: x
					.target_paths
					.into_iter()
					.map(|x| x.to_string_lossy().to_string())
					.collect(),
				kind: x.kind,
				hashes: x.hashes,
			})
			.collect();

		// Combine bundled and included dependencies from the package with the results from the modpack
		let packages = result.packages;
		let packages = merge_package_lists(
			download_result.bundled.into_iter().map(|x| x.to_string()),
			&packages,
		);
		let packages = merge_package_lists(
			download_result.included.into_iter().map(|x| x.to_string()),
			&packages,
		);

		let lockfile_modpack = LockfileModpack {
			name: result.name,
			path: modpack_path_str,
			packages,
		};

		let files_to_remove = inst_lock.update_modpack(lockfile_modpack, &addons);
		for file in files_to_remove {
			let file = PathBuf::from(file);
			if file.exists() {
				std::fs::remove_file(file)?;
			}
		}
		inst_lock.write().context("Failed to write lockfile")?;

		Ok(ModpackInstallResult::default())
	}

	/// Import an instance by downloading and installing a modpack
	pub async fn create_from_modpack_package(
		id: &str,
		modpack: &ArcPkgReq,
		side: Side,
		version_list: Vec<String>,
		reg: &PkgRegistry,
		plugins: &PluginManager,
		client: &Client,
		paths: &Paths,
		o: &mut impl NitroOutput,
	) -> anyhow::Result<InstanceConfig> {
		let mut process = o.get_process();
		process.display(MessageContents::StartProcess("Downloading modpack".into()));

		let constants = EvalConstants {
			version: None,
			loader: Loader::Any,
			version_list: version_list.clone(),
			language: Language::default(),
			default_stability: PackageStability::Latest,
			suppress: Vec::new(),
		};
		let params = EvalParameters::new(side);

		let input = EvalInput {
			constants: Arc::new(constants),
			params,
		};

		let download_result = download_modpack_package(
			modpack,
			input,
			id,
			reg,
			plugins,
			client,
			paths,
			process.deref_mut(),
		)
		.await?;

		let Some(download_result) = download_result else {
			bail!("Modpack package did not return an addon");
		};

		process.display(MessageContents::Success("Modpack downloaded".into()));
		process.finish();

		// Install with instance transfer

		let modpack_formats = plugins
			.call_hook(AddModpackFormats, &(), paths, o)
			.await?
			.flatten_all_results_with_ids(o)
			.await?;

		let Some((_, modpack_format)) = modpack_formats
			.into_iter()
			.find(|x| x.1.id == download_result.format)
		else {
			bail!(
				"Modpack format {} is not supported. Try installing a plugin for it.",
				download_result.format
			);
		};

		let Some(transfer_format) = modpack_format.transfer_format else {
			bail!("Modpack format does not support importing");
		};

		let transfer_formats = load_formats(plugins, paths, o)
			.await
			.context("Failed to load transfer formats")?;

		let mut config = Self::import(
			id,
			&transfer_format,
			&download_result.modpack_path,
			Some(side),
			&transfer_formats,
			&version_list,
			plugins,
			paths,
			o,
		)
		.await
		.context("Failed to install modpack")?;

		config.modpack = Some(modpack.to_string());

		// Download and add the icon
		if let Some(icon) = download_result.icon {
			let pkg_identifier = make_valid_instance_id(&modpack.to_string_no_version());
			let path = paths.internal.join("icons").join(&pkg_identifier);
			if !path.exists() {
				let _ = create_leading_dirs_async(&path).await;
				if let Err(e) = download::file(&icon, &path, client).await {
					o.display(MessageContents::Error(format!(
						"Failed to download modpack icon: {e:?}"
					)));
				}
			}

			config.icon = Some(path.to_string_lossy().to_string());
		}

		Ok(config)
	}
}

/// Result from updating modpack installation
#[derive(Default, Clone)]
pub struct ModpackInstallResult {
	/// Packages provided by the modpack
	pub supplied_packages: Vec<String>,
}

/// Evaluates a modpack package and downloads its addon
pub async fn download_modpack_package(
	modpack: &ArcPkgReq,
	input: EvalInput,
	instance_id: &str,
	reg: &PkgRegistry,
	plugins: &PluginManager,
	client: &Client,
	paths: &Paths,
	o: &mut impl NitroOutput,
) -> anyhow::Result<Option<ModpackDownloadResult>> {
	// Evaluate the package
	let package = reg
		.get(modpack, paths, client, o)
		.await
		.context("Failed to get modpack package")?;

	let result = package
		.eval(paths, Routine::Install, input, client, plugins.clone())
		.await
		.context("Failed to evaluate modpack")?;

	let Some(addon) = result.addon_reqs.first() else {
		o.debug(MessageContents::Simple(
			"No modpack addon was returned".into(),
		));
		o.display(MessageContents::Success("Modpack installed".into()));
		return Ok(None);
	};

	if addon.addon.kind != AddonKind::Modpack {
		bail!("Modpack package did not have a modpack addon");
	}

	let Some(format) = &addon.addon.modpack_format else {
		bail!("Modpack addon did not specify a format");
	};

	// Download the modpack
	let modpack_path = addon.addon.get_path(paths, instance_id);
	if !modpack_path.exists() {
		addon
			.acquire(paths, instance_id, client)
			.await
			.context("Failed to download modpack")?;
	}

	let meta = package
		.get_metadata(paths, client)
		.await
		.context("Failed to get modpack metadata")?;

	Ok(Some(ModpackDownloadResult {
		modpack_path,
		format: format.clone(),
		icon: meta.icon.clone(),
		bundled: result.bundled,
		included: result.inclusions,
	}))
}

/// Result from downloading a modpack package
#[derive(Default, Clone)]
pub struct ModpackDownloadResult {
	/// Path to the modpack file
	pub modpack_path: PathBuf,
	/// Format of the modpack
	pub format: String,
	/// URL to the icon of the modpack
	pub icon: Option<String>,
	/// Packages bundled with the modpack, to be included in suppression
	pub bundled: Vec<Arc<str>>,
	/// Packages included with the modpack, to be included in suppression
	pub included: Vec<Arc<str>>,
}
