use std::{collections::HashMap, sync::Arc};

use super::CmdData;
use itertools::Itertools;
use mcvm::data::id::ProfileID;
use mcvm::parse::lex::Token;
use mcvm::pkg_crate::{PackageContentType, PkgRequest, PkgRequestSource};
use mcvm::util::print::ReplPrinter;

use anyhow::{bail, Context};
use clap::Subcommand;
use color_print::{cformat, cprint, cprintln};
use mcvm::shared::pkg::PackageID;
use reqwest::Client;

use crate::output::HYPHEN_POINT;

#[derive(Debug, Subcommand)]
pub enum PackageSubcommand {
	#[command(about = "List all installed packages across all profiles")]
	#[clap(alias = "ls")]
	List {
		/// Whether to remove formatting and warnings from the output
		#[arg(short, long)]
		raw: bool,
		/// A profile to filter packages from
		#[arg(short, long)]
		profile: Option<String>,
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

pub async fn run(subcommand: PackageSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		PackageSubcommand::List { raw, profile } => list(data, raw, profile).await,
		PackageSubcommand::Sync { filter } => sync(data, filter).await,
		PackageSubcommand::Cat { raw, package } => cat(data, &package, raw).await,
		PackageSubcommand::Info { package } => info(data, &package).await,
		PackageSubcommand::Repository { command } => repo(command, data).await,
	}
}

async fn list(data: &mut CmdData, raw: bool, profile: Option<String>) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	if let Some(profile_id) = profile {
		let profile_id = ProfileID::from(profile_id);
		if let Some(profile) = config.profiles.get(&profile_id) {
			if raw {
				for pkg in profile
					.packages
					.iter_global()
					.sorted_by_key(|x| x.get_pkg_id())
				{
					println!("{}", pkg);
				}
			} else {
				cprintln!("<s>Packages in profile <b>{}</b>:", profile_id);
				for pkg in profile
					.packages
					.iter_global()
					.sorted_by_key(|x| x.get_pkg_id())
				{
					cprintln!("{}<b!>{}</>", HYPHEN_POINT, pkg);
				}
			}
		} else {
			bail!("Unknown profile '{profile_id}'");
		}
	} else {
		let mut found_pkgs: HashMap<PackageID, Vec<ProfileID>> = HashMap::new();
		for (id, profile) in config.profiles.iter() {
			for pkg in profile.packages.iter_global() {
				found_pkgs
					.entry(pkg.get_pkg_id().clone())
					.or_insert(vec![])
					.push(id.clone());
			}
		}
		if raw {
			for (pkg, ..) in found_pkgs.iter().sorted_by_key(|x| x.0) {
				println!("{pkg}");
			}
		} else {
			cprintln!("<s>Packages:");
			for (pkg, profiles) in found_pkgs.iter().sorted_by_key(|x| x.0) {
				cprintln!("<b!>{}</>", pkg);
				for profile in profiles.iter().sorted() {
					cprintln!("{}<k!>{}", HYPHEN_POINT, profile);
				}
			}
		}
	}

	Ok(())
}

async fn sync(data: &mut CmdData, filter: Vec<String>) -> anyhow::Result<()> {
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
		.update_cached_packages(&data.paths, &client, &mut data.output)
		.await
		.context("Failed to update cached packages")?;
	printer.println(&cformat!("<s>Validating packages..."));
	let client = Client::new();
	for package in config.packages.get_all_packages() {
		match config
			.packages
			.parse_and_validate(&package, &data.paths, &client, &mut data.output)
			.await
		{
			Ok(..) => {}
			Err(e) => printer.println(&cformat!(
				"<y>Warning: Package '{}' was invalid:\n{:?}",
				package,
				e
			)),
		}
	}

	Ok(())
}

async fn cat(data: &mut CmdData, id: &str, raw: bool) -> anyhow::Result<()> {
	data.ensure_config(!raw).await?;
	let config = data.config.get_mut();

	let client = Client::new();

	let req = Arc::new(PkgRequest::parse(id, PkgRequestSource::UserRequire));
	let contents = config
		.packages
		.load(&req, &data.paths, &client, &mut data.output)
		.await?;
	if !raw {
		cprintln!("<s,b>Contents of package <g>{}</g>:</s,b>", req);
	}

	if raw {
		print!("{contents}");
	} else {
		let content_type = config
			.packages
			.content_type(&req, &data.paths, &client, &mut data.output)
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
	dbg!(&lexed, contents.len());

	let mut last_tok_was_at = false;
	let mut last_tok_was_curly_or_semi = false;
	for elem in lexed.windows(2) {
		if let Some(left) = elem.get(0) {
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

async fn info(data: &mut CmdData, id: &str) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let client = Client::new();

	let req = Arc::new(PkgRequest::parse(id, PkgRequestSource::UserRequire));
	let metadata = config
		.packages
		.get_metadata(&req, &data.paths, &client, &mut data.output)
		.await
		.context("Failed to get metadata from the registry")?;
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
			cprintln!("   {}", long_description);
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

async fn repo(subcommand: RepoSubcommand, data: &mut CmdData) -> anyhow::Result<()> {
	match subcommand {
		RepoSubcommand::List { raw } => repo_list(data, raw).await,
		RepoSubcommand::Info { repo } => repo_info(data, repo).await,
	}
}

async fn repo_list(data: &mut CmdData, raw: bool) -> anyhow::Result<()> {
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

async fn repo_info(data: &mut CmdData, repo_id: String) -> anyhow::Result<()> {
	data.ensure_config(true).await?;
	let config = data.config.get_mut();

	let repo = config.packages.repos.iter_mut().find(|x| x.id == repo_id);
	let Some(repo) = repo else {
		bail!("Repository {repo_id} does not exist");
	};

	// Get the repo package count and metadata
	let client = Client::new();

	let pkg_count = repo
		.get_package_count(&data.paths, &client, &mut data.output)
		.await
		.context("Failed to get repository package count")?;

	let meta = repo
		.get_metadata(&data.paths, &client, &mut data.output)
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