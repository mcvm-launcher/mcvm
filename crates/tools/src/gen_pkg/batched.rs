use std::{fs::File, io::BufWriter, path::PathBuf};

use reqwest::Client;
use serde::Deserialize;

use crate::gen_pkg::json_merge;

use super::{PackageGenerationConfig, PackageSource};

/// Configuration for a lot of package generation
#[derive(Deserialize)]
pub struct BatchedConfig {
	/// The packages to generate
	pub packages: Vec<BatchedPackageConfig>,
	/// The output directory for the packages
	pub output_dir: String,
}

/// Configuration for a single batched package generation
#[derive(Deserialize)]
pub struct BatchedPackageConfig {
	/// The source for the package
	pub source: PackageSource,
	/// The ID of the package at the source
	pub id: String,
	/// The ID of the generated package
	pub pkg_id: String,
	/// The config for the generated package
	#[serde(flatten)]
	pub config: PackageGenerationConfig,
}

/// Generate a lot of packages
pub async fn batched_gen(config: BatchedConfig) {
	let client = Client::new();

	// Collect Modrinth projects and versions
	let modrinth_ids: Vec<_> = config
		.packages
		.iter()
		.filter(|x| x.source == PackageSource::Modrinth)
		.map(|x| x.id.clone())
		.collect();
	let modrinth_projects = mcvm::net::modrinth::get_multiple_projects(&modrinth_ids, &client)
		.await
		.expect("Failed to get Modrinth projects");
	let modrinth_version_ids: Vec<_> = modrinth_projects
		.iter()
		.flat_map(|x| x.versions.iter().cloned())
		.collect();
	let modrinth_versions =
		mcvm::net::modrinth::get_multiple_versions(&modrinth_version_ids, &client)
			.await
			.expect("Failed to get Modrinth versions");

	// Collect Modrinth teams
	let mut modrinth_team_ids = Vec::new();
	for project in &modrinth_projects {
		modrinth_team_ids.push(project.team.clone());
	}
	let modrinth_teams = mcvm::net::modrinth::get_multiple_teams(&modrinth_team_ids, &client)
		.await
		.expect("Failed to get Modrinth teams");

	// Iterate through the packages to generate
	for pkg in config.packages {
		let package = match pkg.source {
			PackageSource::Smithed => {
				// Just generate the package
				super::smithed::gen(
					&pkg.id,
					pkg.config.relation_substitutions,
					&pkg.config.force_extensions,
				)
				.await;
			}
			PackageSource::Modrinth => {
				// Get the project
				let project = modrinth_projects
					.iter()
					.find(|x| x.id == pkg.id)
					.expect("Project should have been fetched");
				// Get the team associated with this project
				let team = modrinth_teams
					.iter()
					.find(|team| team.iter().any(|member| member.team_id == project.team))
					.expect("Team should have been fetched");
				super::modrinth::gen_raw(
					project.clone(),
					&modrinth_versions,
					team,
					pkg.config.relation_substitutions,
					&pkg.config.force_extensions,
				)
				.await;
			}
		};

		// Merge with config
		let mut package =
			serde_json::value::to_value(package).expect("Failed to convert package to value");
		let merge = serde_json::value::to_value(pkg.config.merge)
			.expect("Failed to convert merged config to value");
		json_merge(&mut package, merge);

		// Write out the package
		let path = PathBuf::from(&config.output_dir).join(format!("{}.json", pkg.pkg_id));
		let file =
			BufWriter::new(File::create(path).expect("Failed to create package output file"));
		serde_json::to_writer_pretty(file, &package).expect("Failed to write package to file");
	}
}
