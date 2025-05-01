use std::sync::Mutex;
use std::{collections::HashMap, sync::Arc};

use super::CmdData;
use itertools::Itertools;
use mcvm::config::modifications::{apply_modifications_and_write, ConfigModification};
use mcvm::config::package::PackageConfigDeser;
use mcvm::parse::lex::Token;
use mcvm::pkg_crate::metadata::PackageMetadata;
use mcvm::pkg_crate::properties::PackageProperties;
use mcvm::pkg_crate::{parse_and_validate, PackageContentType, PkgRequest, PkgRequestSource};
use mcvm::shared::id::{InstanceID, ProfileID};
use mcvm::shared::util::print::ReplPrinter;

use anyhow::{bail, Context};
use clap::Subcommand;
use color_print::{cformat, cprint, cprintln};
use mcvm::shared::pkg::PackageID;
use rayon::prelude::*;
use reqwest::Client;
use serde::Serialize;

use crate::commands::instance::pick_instance;
use crate::output::HYPHEN_POINT;

#[derive(Debug, Subcommand)]
pub enum PackageSubcommand {
	#[command(about = "List all installed packages across all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// An instance to filter packages from
		#[arg(short, long)]
		instance: Option<String>,
	},
	#[command(
		about = "Sync package indexes with ones from package repositories",
		long_about = "Sync all package indexes from remote repositories. They will be
cached locally, but all currently cached package scripts will be removed"
	)]
	Sync {
		/// Only sync the repositories that you specify
		#[arg(short, long)]
		filter: Vec<String>,
	},
	#[command(
		about = "Print the contents of a package to standard out",
		long_about = "Print the contents of any package to standard out.
This package does not need to be installed, it just has to be in the index."
	)]
	#[clap(alias = "print")]
	Cat {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// The package to print
		package: String,
	},
	#[command(about = "Print information about a specific package")]
	Info {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// The package to get info about
		package: String,
	},
	#[command(about = "Query information about configured packages repositories")]
	#[clap(alias = "repo")]
	Repository {
		/// The repository subcommand
		#[command(subcommand)]
		command: RepoSubcommand,
	},
	#[command(about = "List available packages from all repositories")]
	ListAll {},
	#[command(about = "Browse packages from the remote repositories")]
	Browse {},
	#[command(about = "Add a package to an instance")]
	Add {
		/// The package to add to the instance
		package: Option<String>,
		/// The instance to add a package to
		instance: Option<String>,
	},
}

#[derive(Debug, Subcommand)]
pub enum RepoSubcommand {
	#[command(about = "List all configured package repositories")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
	},
	#[command(about = "Print information about a specific repository")]
	Info {
		/// The repository to get info about
		repo: String,
	},
}

pub async fn run(subcommand: PackageSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match subcommand {
		PackageSubcommand::List { raw, instance } => list(data, raw, instance).await,
		PackageSubcommand::Sync { filter } => sync(data, filter).await,
		PackageSubcommand::Cat { raw, package } => cat(data, &package, raw).await,
		PackageSubcommand::Info { raw, package } => info(data, &package, raw).await,
		PackageSubcommand::Repository { command } => repo(command, data).await,
		PackageSubcommand::ListAll {} => list_all(data).await,
		PackageSubcommand::Browse {} => browse(data).await,
		PackageSubcommand::Add { package, instance } => add(data, package, instance).await,
	}
}

async fn list(data: &mut CmdData<'_>, raw: bool, instance: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	if let Some(instance_id) = instance {
		let instance_id = InstanceID::from(instance_id);
		let instance = config
			.instances
			.get(&instance_id)
			.with_context(|| format!("Unknown instance '{instance_id}'"))?;
		if !raw {
			cprintln!("<s>Packages in instance <b>{}</b>:", instance_id);
		}
		for pkg in instance
			.get_configured_packages()
			.iter()
			.sorted_by_key(|x| &x.id)
		{
			if raw {
				println!("{}", pkg.id);
			} else {
				cprintln!("{}<b!>{}</>", HYPHEN_POINT, pkg.id);
			}
		}
	} else {
		let mut found_pkgs: HashMap<PackageID, Vec<ProfileID>> = HashMap::new();
		for (id, instance) in config.instances.iter() {
			for pkg in instance.get_configured_packages() {
				found_pkgs
					.entry(pkg.id.clone())
					.or_default()
					.push(id.clone());
			}
		}
		if !raw {
			cprintln!("<s>Packages:");
		}
		for (pkg, profiles) in found_pkgs.iter().sorted_by_key(|x| x.0) {
			if raw {
				println!("{pkg}");
			} else {
				cprintln!("<b!>{}</>", pkg);
				for profile in profiles.iter().sorted() {
					cprintln!("{}<k!>{}", HYPHEN_POINT, profile);
				}
			}
		}
	}

	Ok(())
}

async fn sync(data: &mut CmdData<'_>, filter: Vec<String>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let mut printer = ReplPrinter::new(true);
	let client = Client::new();
	for repo in config.packages.repos.iter_mut() {
		// Skip repositories not in the filter
		if !filter.is_empty() && !filter.contains(&repo.id) {
			continue;
		}

		printer.print(&cformat!("Syncing repository <b>{}</b>...", repo.id));
		match repo.sync(&data.paths, &client).await {
			Ok(..) => {
				printer.print(&cformat!("<g>Synced repository <b!>{}</b!>", repo.id));
			}
			Err(e) => {
				printer.println(&cformat!("<r>{}", e));
				printer.print(&cformat!(
					"<r>Failed to sync repository <r!>{}</r!>",
					repo.id
				));
				continue;
			}
		};
		cprintln!();
	}
	printer.print(&cformat!("<s>Updating packages..."));
	config
		.packages
		.update_cached_packages(&data.paths, &client, data.output)
		.await
		.context("Failed to update cached packages")?;
	printer.println(&cformat!("<g>Packages updated."));

	printer.println(&cformat!("<s>Validating packages..."));
	let ids = config.packages.get_all_packages();

	let mut packages = Vec::with_capacity(ids.len());
	for id in ids {
		let contents = config
			.packages
			.load(&id, &data.paths, &client, data.output)
			.await
			.context("Failed to get package contents")?;
		let content_type = config
			.packages
			.get_content_type(&id, &data.paths, &client, data.output)
			.await
			.context("Failed to get package content type")?;
		packages.push((id, contents, content_type));
	}
	let errors = Arc::new(Mutex::new(Vec::new()));
	packages
		.into_par_iter()
		.for_each(|(id, contents, content_type)| {
			if let Err(e) = parse_and_validate(&contents, content_type) {
				errors.lock().expect("Poisoned mutex").push(cformat!(
					"<y>Warning: Package '{}' was invalid:\n{:#?}",
					id,
					e
				));
			}
		});
	for error in errors.lock().expect("Poisoned mutex").iter() {
		printer.println(error);
	}
	printer.print(&cformat!("<g>Packages validated."));

	Ok(())
}

async fn cat(data: &mut CmdData<'_>, id: &str, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	let client = Client::new();

	let req = Arc::new(PkgRequest::parse(id, PkgRequestSource::UserRequire));
	let contents = config
		.packages
		.load(&req, &data.paths, &client, data.output)
		.await?;
	if !raw {
		cprintln!("<s,b>Contents of package <g>{}</g>:</s,b>", req);
	}

	if raw {
		print!("{contents}");
	} else {
		let content_type = config
			.packages
			.content_type(&req, &data.paths, &client, data.output)
			.await?;
		if let PackageContentType::Script = content_type {
			pretty_print_package_script(&contents)?;
		} else {
			print!("{contents}");
		}
	}

	Ok(())
}

/// Pretty-print a package script
fn pretty_print_package_script(contents: &str) -> anyhow::Result<()> {
	let mut lexed = mcvm::parse::lex::lex(contents)?;

	// Since the windows iterator won't go to the end with the last token on the left
	// side of the window, because it always makes sure the array is at least 2 elements long,
	// we pad the end with a none token
	if let Some(last) = lexed.last().cloned() {
		lexed.push((Token::None, last.1));
	}

	let mut last_tok_was_at = false;
	let mut last_tok_was_curly_or_semi = false;
	for elem in lexed.windows(2) {
		if let Some(left) = elem.first() {
			// If the right does not exist, set it to the end of the string
			let right_pos = elem
				.get(1)
				.map(|x| *x.1.absolute())
				.unwrap_or(contents.len() - 1);

			let left_pos = *left.1.absolute();
			if left_pos >= contents.len() || right_pos >= contents.len() {
				continue;
			}
			// A range thing
			let text = if left_pos == contents.len() - 1 && !contents.is_empty() {
				&contents[left_pos..]
			} else {
				&contents[left_pos..right_pos]
			};
			let text = match left.0 {
				Token::None => String::new(),
				Token::Whitespace => text.to_string(),
				Token::Semicolon
				| Token::Colon
				| Token::Comma
				| Token::Pipe
				| Token::Bang
				| Token::Square(..)
				| Token::Paren(..)
				| Token::Angle(..)
				| Token::Curly(..) => text.to_string(),
				Token::At => cformat!("<m><s>{text}"),
				Token::Variable(..) => cformat!("<c>{text}"),
				Token::Comment(..) => cformat!("<k!>{text}"),
				Token::Ident(..) => {
					if last_tok_was_at {
						cformat!("<m><s>{text}")
					} else if last_tok_was_curly_or_semi {
						cformat!("<b!>{text}")
					} else {
						text.to_string()
					}
				}
				Token::Num(..) => cformat!("<y>{text}"),
				Token::Str(..) => cformat!("<g>{text}"),
			};
			print!("{text}");

			// Whitespace can split these tokens apart so we need to make sure it doesn't
			if !left.0.is_ignored() {
				last_tok_was_at = false;
				last_tok_was_curly_or_semi = false;
			}
			if let Token::At = left.0 {
				last_tok_was_at = true;
			}
			if let Token::Curly(..) | Token::Semicolon = left.0 {
				last_tok_was_curly_or_semi = true;
			}
		}
	}

	Ok(())
}

async fn info(data: &mut CmdData<'_>, id: &str, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let client = Client::new();

	let req = Arc::new(PkgRequest::parse(id, PkgRequestSource::UserRequire));
	let metadata = config
		.packages
		.get_metadata(&req, &data.paths, &client, data.output)
		.await
		.context("Failed to get metadata from the registry")?;

	if raw {
		let metadata = metadata.clone();

		let properties = config
			.packages
			.get_properties(&req, &data.paths, &client, data.output)
			.await
			.context("Failed to get package properties from the registry")?;

		#[derive(Serialize)]
		struct RawOutput<'a> {
			metadata: PackageMetadata,
			properties: &'a PackageProperties,
		}

		let out = serde_json::to_string(&RawOutput {
			metadata,
			properties,
		})
		.context("Failed to serialize raw output")?;

		print!("{out}");

		return Ok(());
	}

	if let Some(name) = &metadata.name {
		cprintln!("<s><g>Package</g> <b>{}</b>", name);
	} else {
		cprintln!("<s><g>Package</g> <b>{}</b>", id);
	}
	if let Some(description) = &metadata.description {
		if !description.is_empty() {
			cprintln!("   <s>{}", description);
		}
	}
	if let Some(long_description) = &metadata.long_description {
		if !long_description.is_empty() {
			termimad::print_text(long_description);
		}
	}
	cprintln!("   <s>ID:</s> <g>{}", id);
	if let Some(authors) = &metadata.authors {
		if !authors.is_empty() {
			cprintln!("   <s>Authors:</s> <g>{}", authors.join(", "));
		}
	}
	if let Some(maintainers) = &metadata.package_maintainers {
		if !maintainers.is_empty() {
			cprintln!(
				"   <s>Package Maintainers:</s> <g>{}",
				maintainers.join(", ")
			);
		}
	}
	if let Some(website) = &metadata.website {
		if !website.is_empty() {
			cprintln!("   <s>Website:</s> <b!>{}", website);
		}
	}
	if let Some(support_link) = &metadata.support_link {
		if !support_link.is_empty() {
			cprintln!("   <s>Support Link:</s> <b!>{}", support_link);
		}
	}
	if let Some(documentation) = &metadata.documentation {
		if !documentation.is_empty() {
			cprintln!("   <s>Documentation:</s> <b!>{}", documentation);
		}
	}
	if let Some(source) = &metadata.source {
		if !source.is_empty() {
			cprintln!("   <s>Source:</s> <b!>{}", source);
		}
	}
	if let Some(issues) = &metadata.issues {
		if !issues.is_empty() {
			cprintln!("   <s>Issue Tracker:</s> <b!>{}", issues);
		}
	}
	if let Some(community) = &metadata.community {
		if !community.is_empty() {
			cprintln!("   <s>Community Link:</s> <b!>{}", community);
		}
	}
	if let Some(license) = &metadata.license {
		if !license.is_empty() {
			cprintln!("   <s>License:</s> <b!>{}", license);
		}
	}

	Ok(())
}

async fn repo(subcommand: RepoSubcommand, data: &mut CmdData<'_>) -> anyhow::Result<()> {
	match subcommand {
		RepoSubcommand::List { raw } => repo_list(data, raw).await,
		RepoSubcommand::Info { repo } => repo_info(data, repo).await,
	}
}

async fn repo_list(data: &mut CmdData<'_>, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	let repos = config.packages.get_repos();

	if raw {
		for repo in repos {
			println!("{}", repo.id);
		}
	} else {
		cprintln!("<s>Repositories:");
		for repo in repos {
			if repo.id == "core" {
				cprint!("<s><m>{}</></>", repo.id);
			} else if repo.id == "std" {
				cprint!("<s><b>{}</></>", repo.id);
			} else {
				cprint!("<s>{}</>", repo.id);
			}
			cprintln!(" <k!>-</> <m>{}</>", repo.get_location());
		}
	}

	Ok(())
}

async fn repo_info(data: &mut CmdData<'_>, repo_id: String) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let repo = config.packages.repos.iter_mut().find(|x| x.id == repo_id);
	let Some(repo) = repo else {
		bail!("Repository {repo_id} does not exist");
	};

	// Get the repo package count and metadata
	let client = Client::new();

	let pkg_count = repo
		.get_package_count(&data.paths, &client, data.output)
		.await
		.context("Failed to get repository package count")?;

	let meta = repo
		.get_metadata(&data.paths, &client, data.output)
		.await
		.context("Failed to get repository metadata")?
		.as_ref()
		.clone();

	// Print the repo name
	let name = if let Some(name) = &meta.name {
		name.clone()
	} else {
		repo.id.clone()
	};

	cprint!("<s>Repository </>");
	if repo.id == "core" {
		cprint!("<s><m>{}</></>", name);
	} else if repo.id == "std" {
		cprint!("<s><b>{}</></>", name);
	} else {
		cprint!("<s>{}</>", name);
	}
	cprintln!("<s>:</>");

	// Print info
	if let Some(description) = &meta.description {
		cprintln!("   {}", description);
	}
	cprintln!("   <s>ID:</> {}", repo.id);
	cprintln!("   <s>Location:</> <m>{}</>", repo.get_location());
	if let Some(version) = &meta.mcvm_version {
		cprintln!("   <s>MCVM Version:</> <c>{}</>", version);
	}
	cprintln!("   <s>Package Count:</> <y>{}</>", pkg_count);

	Ok(())
}

async fn list_all(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let client = Client::new();
	let mut packages = config
		.packages
		.get_all_available_packages(&data.paths, &client, data.output)
		.await
		.context("Failed to get list of available packages")?;
	packages.sort();

	for package in packages {
		println!("{package}");
	}

	Ok(())
}

async fn browse(data: &mut CmdData<'_>) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let client = Client::new();
	let mut packages = config
		.packages
		.get_all_available_packages(&data.paths, &client, data.output)
		.await
		.context("Failed to get list of available packages")?;
	packages.sort();

	loop {
		let select =
			inquire::Select::new("Browse packages. Press Escape to exit.", packages.clone());
		let package = select.prompt_skippable()?;
		if let Some(package) = package {
			info(data, &package.id, false).await?;
			inquire::Confirm::new("Press Escape to return to browse page").prompt_skippable()?;
		} else {
			break;
		}
	}

	Ok(())
}

async fn add(
	data: &mut CmdData<'_>,
	package: Option<String>,
	instance: Option<String>,
) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let client = Client::new();
	let mut packages = config
		.packages
		.get_all_available_packages(&data.paths, &client, data.output)
		.await
		.context("Failed to get list of available packages")?;
	packages.sort();

	let package = if let Some(package) = package {
		Arc::from(package)
	} else {
		inquire::Select::new("Which package would you like to install?", packages)
			.prompt()
			.context("Failed to get desired package")?
			.id
			.clone()
	};

	let instance =
		pick_instance(instance, config).context("Failed to get instance to add package to")?;

	let mut config_raw = data.get_raw_config()?;
	apply_modifications_and_write(
		&mut config_raw,
		vec![ConfigModification::AddPackage(
			instance,
			PackageConfigDeser::Basic(package),
		)],
		&data.paths,
	)
	.context("Failed to write modified config")?;

	cprintln!("<g>Package added.");

	Ok(())
}
