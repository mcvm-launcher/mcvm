use super::lib::{CmdData, CmdError};
use crate::data::addon::PluginLoader;
use crate::io::lock::Lockfile;
use crate::io::lock::LockfileAddon;
use crate::net::game_files::get_version_manifest;
use crate::net::game_files::make_version_list;
use crate::net::paper;
use crate::package::eval::eval::Routine;
use crate::package::eval::eval::EvalConstants;
use crate::data::instance::InstKind;
use crate::util::print::HYPHEN_POINT;
use crate::util::print::ReplPrinter;

use color_print::cformat;
use color_print::{cprintln, cprint};

static INFO_HELP: &str = "View helpful information about a profile";
static LIST_HELP: &str = "List all profiles and their instances";
static UPDATE_HELP: &str = "Update the packages and instances of a profile";
static REINSTALL_HELP: &str = "Force reinstall a profile and all its files";

pub fn help() {
	cprintln!("<i>profile:</i> Manage mcvm profiles");
	cprintln!("<s>Usage:</s> mcvm profile <k!><<subcommand>> [options]</k!>");
	cprintln!();
	cprintln!("<s>Subcommands:");
	cprintln!("{}<i,c>info:</i,c> {}", HYPHEN_POINT, INFO_HELP);
	cprintln!("{}<i,c>list, ls:</i,c> {}", HYPHEN_POINT, LIST_HELP);
	cprintln!("{}<i,c>update:</i,c> {}", HYPHEN_POINT, UPDATE_HELP);
	cprintln!("{}<i,c>reinstall:</i,c> {}", HYPHEN_POINT, REINSTALL_HELP);
}

fn info(data: &mut CmdData, id: &String) -> Result<(), CmdError> {
	data.ensure_paths()?;
	data.ensure_config()?;

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
							InstKind::Client => cprint!("<y!>Client {}", inst_id),
							InstKind::Server => cprint!("<c!>Server {}", inst_id)
						}
						cprintln!();
					}
				}
				cprintln!("   <s>Packages:");
				for pkg in profile.packages.iter() {
					cprint!("   {}", HYPHEN_POINT);
					cprint!("<b!>{}:<g!>{}", pkg.req.name, config.packages.get_version(&pkg.req, paths)?);
					cprintln!();
				}
			} else {
				return Err(CmdError::Custom(format!("Unknown profile '{id}'")));
			}
		}
	}
	Ok(())
}

fn list(data: &mut CmdData) -> Result<(), CmdError> {
	data.ensure_config()?;

	if let Some(config) = &data.config {
		cprintln!("<s>Profiles:");
		for (id, profile) in config.profiles.iter() {
			cprintln!("<s><g>   {}", id);
			for inst_id in profile.instances.iter() {
				if let Some(instance) = config.instances.get(inst_id) {
					match instance.kind {
						InstKind::Client => cprintln!("   {}<y!>{}", HYPHEN_POINT, inst_id),
						InstKind::Server => cprintln!("   {}<c!>{}", HYPHEN_POINT, inst_id)
					}
				}
			}
		}
	}
	Ok(())
}

async fn profile_update(data: &mut CmdData, id: &String, force: bool) -> Result<(), CmdError> {
	data.ensure_paths()?;
	data.ensure_config()?;
	
	if let Some(config) = &mut data.config {
		if let Some(paths) = &data.paths {
			if let Some(profile) = config.profiles.get_mut(id) {
				let (paper_build_num, paper_file_name) = if let PluginLoader::Paper = profile.plugin_loader {
					let (build_num, ..) = paper::get_newest_build(&profile.version).await?;
					let paper_file_name = paper::get_jar_file_name(&profile.version, build_num).await?;
					(Some(build_num), Some(paper_file_name))
				} else {
					(None, None)
				};
				let mut lock = Lockfile::open(paths)?;
				if lock.update_profile_version(id, &profile.version) {
					cprintln!("<s>Updating profile version...");
					for inst in profile.instances.iter() {
						if let Some(inst) = config.instances.get(inst) {
							inst.teardown(paths, paper_file_name.clone())?;
						}
					}
				}
				if let Some(build_num) = paper_build_num {
					if let Some(file_name) = paper_file_name {
						if lock.update_profile_paper_build(id, build_num) {
							for inst in profile.instances.iter() {
								if let Some(inst) = config.instances.get(inst) {
									inst.remove_paper(paths, file_name.clone())?;
								}
							}
						}
					}
				}

				lock.finish(paths)?;

				cprintln!("<s>Obtaining version index...");
				let (version_manifest, ..) = get_version_manifest(paths)?;
				profile.create_instances(&mut config.instances, &version_manifest, paths, true, force).await?;
				
				cprintln!("<s>Updating packages");
				let mut printer = ReplPrinter::new(true);
				for pkg in profile.packages.iter() {
					let version = config.packages.get_version(&pkg.req, paths)?;
					for instance_id in profile.instances.iter() {
						if let Some(instance) = config.instances.get(instance_id) {
							printer.print(&cformat!("\t(<b!>{}</b!>) Evaluating...", pkg.req));
							let constants = EvalConstants {
								version: profile.version.clone(),
								modloader: profile.modloader.clone(),
								plugin_loader: profile.plugin_loader.clone(),
								side: instance.kind.clone(),
								features: pkg.features.clone(),
								versions: make_version_list(&version_manifest)?,
								perms: pkg.permissions.clone()
							};
							let eval = config.packages.eval(&pkg.req, paths, Routine::Install, constants).await?;
							printer.print(&cformat!("\t(<b!>{}</b!>) Downloading files...", pkg.req));
							for addon in eval.addon_reqs.iter() {
								addon.acquire(paths).await?;
								instance.create_addon(&addon.addon, paths)?;
							}
							let lockfile_addons = eval.addon_reqs.iter().map(|x| {
								LockfileAddon::from_addon(&x.addon, paths)
							}).collect::<Vec<LockfileAddon>>();
							let addons_to_remove = lock.update_package(&pkg.req.name, instance_id, &version, &lockfile_addons)?;
							for addon in eval.addon_reqs.iter() {
								if addons_to_remove.contains(&addon.addon.name) {
									instance.remove_addon(&addon.addon, paths)?;
								}
							}

							printer.newline();
						}
					}
				}
				
				for instance_id in profile.instances.iter() {
					if let Some(instance) = config.instances.get(instance_id) {
						let addons_to_remove = lock.remove_unused_packages(
							instance_id,
							&profile.packages.iter().map(|x| x.req.name.clone())
								.collect::<Vec<String>>()
						)?;

						for addon in addons_to_remove {
							instance.remove_addon(&addon, paths)?;
						}
					}
				}

				lock.finish(paths)?;

				printer.print(&cformat!("\t<g>Finished installing packages."));
				printer.finish();
			} else {
				return Err(CmdError::Custom(format!("Unknown profile '{id}'")));
			}
		}
	}
	Ok(())
}

pub async fn run(argc: usize, argv: &[String], data: &mut CmdData)
-> Result<(), CmdError> {
	if argc == 0 {
		help();
		return Ok(());
	}

	match argv[0].as_str() {
		"list" | "ls" => list(data)?,
		"info" => match argc {
			1 => cprintln!("{}", INFO_HELP),
			_ => info(data, &argv[1])?
		}
		"update" => match argc {
			1 => cprintln!("{}", UPDATE_HELP),
			_ => profile_update(data, &argv[1], false).await?
		}
		"reinstall" => match argc {
			1 => cprintln!("{}", REINSTALL_HELP),
			_ => profile_update(data, &argv[1], true).await?
		}
		cmd => cprintln!("<r>Unknown subcommand {}", cmd)
	}

	Ok(())
}
