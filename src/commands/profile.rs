use super::CmdData;
use crate::data::addon::PluginLoader;
use crate::data::instance::InstKind;
use crate::data::profile::update::UpdateManager;
use crate::io::lock::Lockfile;
use crate::io::lock::LockfileAddon;
use crate::net::paper;
use crate::package::eval::eval::EvalConstants;
use crate::package::eval::eval::Routine;
use crate::util::print::PrintOptions;
use crate::util::print::ReplPrinter;
use crate::util::print::HYPHEN_POINT;

use anyhow::bail;
use anyhow::Context;
use clap::Subcommand;
use color_print::cformat;
use color_print::{cprint, cprintln};

#[derive(Debug, Subcommand)]
pub enum ProfileSubcommand {
	#[command(about = "Print useful information about a profile")]
	Info { profile: String },
	#[command(about = "List all profiles")]
	#[clap(alias = "ls")]
	List,
	#[command(
		about = "Update a profile",
		long_about = "Update the game files, extensions, packages, and addons of a profile."
	)]
	Update {
		/// Whether to force update files that have already been downloaded
		#[arg(short, long)]
		force: bool,
		/// The profile to update
		profile: String,
	},
}

async fn info(data: &mut CmdData, id: &str) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config().await?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(profile) = config.profiles.get(id) {
				cprintln!("<s><g>Profile <b>{}", id);
				cprintln!("   <s>Version:</s> <g>{}", profile.version);
				cprintln!("   <s>Modloader:</s> <g>{}", profile.modloader);
				cprintln!("   <s>Plugin Loader:</s> <g>{}", profile.plugin_loader);
				cprintln!("   <s>Instances:");
				for inst_id in profile.instances.iter() {
					if let Some(instance) = config.instances.get(inst_id) {
						cprint!("   {}", HYPHEN_POINT);
						match instance.kind {
							InstKind::Client { .. } => cprint!("<y!>Client {}", inst_id),
							InstKind::Server { .. } => cprint!("<c!>Server {}", inst_id),
						}
						cprintln!();
					}
				}
				cprintln!("   <s>Packages:");
				for pkg in profile.packages.iter() {
					cprint!("   {}", HYPHEN_POINT);
					cprint!(
						"<b!>{}:<g!>{}",
						pkg.req.name,
						config
							.packages
							.get_version(&pkg.req, paths)
							.await
							.context("Failed to get package version")?
					);
					cprintln!();
				}
			} else {
				bail!("Unknown profile '{id}'");
			}
		}
	}
	Ok(())
}

async fn list(data: &mut CmdData) -> anyhow::Result<()> {
	data.ensure_config().await?;

	if let Some(config) = &data.config {
		cprintln!("<s>Profiles:");
		for (id, profile) in config.profiles.iter() {
			cprintln!("<s><g>   {}", id);
			for inst_id in profile.instances.iter() {
				if let Some(instance) = config.instances.get(inst_id) {
					match instance.kind {
						InstKind::Client { .. } => cprintln!("   {}<y!>{}", HYPHEN_POINT, inst_id),
						InstKind::Server { .. } => cprintln!("   {}<c!>{}", HYPHEN_POINT, inst_id),
					}
				}
			}
		}
	}
	Ok(())
}

async fn profile_update(data: &mut CmdData, id: &str, force: bool) -> anyhow::Result<()> {
	data.ensure_paths().await?;
	data.ensure_config().await?;

	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(profile) = config.profiles.get_mut(id) {
				let print_options = PrintOptions::new(true, 0);
				let mut manager = UpdateManager::new(print_options, force);
				manager
					.fulfill_version_manifest(paths, &profile.version)
					.await
					.context("Failed to get version information")?;
				let version = manager.found_version.get();

				let (paper_build_num, paper_file_name) =
					if let PluginLoader::Paper = profile.plugin_loader {
						let (build_num, ..) = paper::get_newest_build(version)
							.await
							.context("Failed to get the newest Paper build number")?;
						let paper_file_name = paper::get_jar_file_name(version, build_num)
							.await
							.context("Failed to get the name of the Paper Jar file")?;
						(Some(build_num), Some(paper_file_name))
					} else {
						(None, None)
					};
				let mut lock = Lockfile::open(paths)
					.await
					.context("Failed to open lockfile")?;
				if lock.update_profile_version(id, version) {
					cprintln!("<s>Updating profile version...");
					for inst in profile.instances.iter() {
						if let Some(inst) = config.instances.get(inst) {
							inst.teardown(paths, paper_file_name.clone()).context(
								"Failed to remove old files when updating Minecraft version",
							)?;
						}
					}
				}
				if let Some(build_num) = paper_build_num {
					if let Some(file_name) = paper_file_name {
						if lock.update_profile_paper_build(id, build_num) {
							for inst in profile.instances.iter() {
								if let Some(inst) = config.instances.get(inst) {
									inst.remove_paper(paths, file_name.clone())
										.context("Failed to remove Paper")?;
								}
							}
						}
					}
				}

				lock.finish(paths)
					.await
					.context("Failed to finish using lockfile")?;

				if !profile.instances.is_empty() {
					let version_list = profile
						.create_instances(&mut config.instances, paths, manager)
						.await
						.context("Failed to create profile instances")?;

					if !profile.packages.is_empty() {
						cprintln!("<s>Updating packages");
					}
					let mut printer = ReplPrinter::new(true);
					for pkg in profile.packages.iter() {
						let version = config
							.packages
							.get_version(&pkg.req, paths)
							.await
							.context("Failed to get version for package")?;
						printer.print(&cformat!("\t(<b!>{}</b!>) Installing...", pkg.req));
						for instance_id in profile.instances.iter() {
							if let Some(instance) = config.instances.get(instance_id) {
								let constants = EvalConstants {
									version: version.clone(),
									modloader: profile.modloader.clone(),
									plugin_loader: profile.plugin_loader.clone(),
									side: instance.kind.to_side(),
									features: pkg.features.clone(),
									versions: version_list.clone(),
									perms: pkg.permissions.clone(),
								};
								let eval = config
									.packages
									.eval(&pkg.req, paths, Routine::Install, constants)
									.await
									.with_context(|| {
										format!(
											"Failed to evaluate package {} for instance {}",
											pkg.req, instance_id
										)
									})?;
								for addon in eval.addon_reqs.iter() {
									addon.acquire(paths).await.with_context(|| {
										format!(
											"Failed to acquire addon {} for instance {}",
											addon.addon.name, instance_id
										)
									})?;
									instance.create_addon(&addon.addon, paths).with_context(
										|| {
											format!(
												"Failed to install addon {} for instance {}",
												addon.addon.name, instance_id
											)
										},
									)?;
								}
								let lockfile_addons = eval
									.addon_reqs
									.iter()
									.map(|x| LockfileAddon::from_addon(&x.addon, paths))
									.collect::<Vec<LockfileAddon>>();
								let addons_to_remove = lock
									.update_package(
										&pkg.req.name,
										instance_id,
										&version,
										&lockfile_addons,
									)
									.context("Failed to update package in lockfile")?;
								for addon in eval.addon_reqs.iter() {
									if addons_to_remove.contains(&addon.addon.name) {
										instance.remove_addon(&addon.addon, paths).with_context(
											|| {
												format!(
													"Failed to remove addon {} for instance {}",
													addon.addon.name, instance_id
												)
											},
										)?;
									}
								}
							}
						}
						printer.print(&cformat!("\t(<b!>{}</b!>) <g>Installed.", pkg.req));
						printer.newline();
					}

					for instance_id in profile.instances.iter() {
						if let Some(instance) = config.instances.get(instance_id) {
							let addons_to_remove = lock
								.remove_unused_packages(
									instance_id,
									&profile
										.packages
										.iter()
										.map(|x| x.req.name.clone())
										.collect::<Vec<String>>(),
								)
								.context("Failed to remove unused packages")?;

							for addon in addons_to_remove {
								instance.remove_addon(&addon, paths).with_context(|| {
									format!(
										"Failed to remove addon {} for instance {}",
										addon.name, instance_id
									)
								})?;
							}
						}
					}
					printer.print(&cformat!("\t<g>Finished installing packages."));
					printer.finish();
				}

				lock.finish(paths)
					.await
					.context("Failed to finish using lockfile")?;
			} else {
				bail!("Unknown profile '{id}'");
			}
		}
	}
	Ok(())
}

pub async fn run(subcommand: ProfileSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		ProfileSubcommand::Info { profile } => info(data, &profile).await,
		ProfileSubcommand::List => list(data).await,
		ProfileSubcommand::Update { force, profile } => profile_update(data, &profile, force).await,
	}
}
