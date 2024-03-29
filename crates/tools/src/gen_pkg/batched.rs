use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

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
	/// A directory to read packages from
	pub config_dir: Option<String>,
}

/// Configuration for a single batched package generation
#[derive(Deserialize)]
pub struct BatchedPackageConfig {
	/// The source for the package
	pub source: PackageSource,
	/// The ID of the package at the source
	pub id: String,
	/// The ID of the generated package
	pub pkg_id: Option<String>,
	/// The config for the generated package
	#[serde(flatten)]
	pub config: PackageGenerationConfig,
}

/// Generate a lot of packages
pub async fn batched_gen(mut config: BatchedConfig) {
	// Read config dir for additional packages
	if let Some(config_dir) = config.config_dir {
		let config_dir = PathBuf::from(config_dir);
		let mut additional_pkgs = Vec::new();
		for entry in std::fs::read_dir(config_dir).expect("Failed to read config directory") {
			let entry = entry.expect("Failed to read config directory entry");
			let file_type = entry
				.file_type()
				.expect("Failed to get config dir entry file type");
			if file_type.is_file() {
				let pkg_id = entry
					.path()
					.file_stem()
					.expect("File stem missing")
					.to_string_lossy()
					.to_string();
				let file = File::open(entry.path()).expect("Failed to open package config file");
				let mut config: BatchedPackageConfig =
					serde_json::from_reader(file).expect("Failed to read package config file");
				config.pkg_id = Some(pkg_id);
				additional_pkgs.push(config);
			}
		}

		config.packages.extend(additional_pkgs);
	}

	let client = Client::new();

	println!("Requesting API...");

	// Collect Modrinth projects
	let modrinth_ids: Vec<_> = config
		.packages
		.iter()
		.filter(|x| x.source == PackageSource::Modrinth)
		.map(|x| x.id.clone())
		.collect();
	let modrinth_projects = mcvm::net::modrinth::get_multiple_projects(&modrinth_ids, &client)
		.await
		.expect("Failed to get Modrinth projects");

	// Collect Modrinth project versions. We have to batch these into multiple requests because there becomes
	// just too many parameters for the URL to handle
	let batch_limit = 200;
	let modrinth_version_ids: Vec<_> = modrinth_projects
		.iter()
		.flat_map(|x| x.versions.iter().cloned())
		.collect();
	let chunks = modrinth_version_ids.chunks(batch_limit);
	let mut modrinth_versions = Vec::new();
	for chunk in chunks {
		modrinth_versions.extend(
			mcvm::net::modrinth::get_multiple_versions(chunk, &client)
				.await
				.expect("Failed to get Modrinth versions"),
		);
	}

	// Collect Modrinth teams
	let mut modrinth_team_ids = Vec::new();
	for project in &modrinth_projects {
		modrinth_team_ids.push(project.team.clone());
	}
	let modrinth_teams = mcvm::net::modrinth::get_multiple_teams(&modrinth_team_ids, &client)
		.await
		.expect("Failed to get Modrinth teams");

	// Iterate through the packages to generate
	println!("Generating packages...");
	for pkg in config.packages {
		println!(
			"Generating package {}",
			pkg.pkg_id.as_ref().expect("Package ID should exist")
		);
		let package = match pkg.source {
			PackageSource::Smithed => {
				// Just generate the package
				super::smithed::gen(
					&pkg.id,
					pkg.config.relation_substitutions,
					&pkg.config.force_extensions,
				)
				.await
			}
			PackageSource::Modrinth => {
				// Get the project
				let project = modrinth_projects
					.iter()
					.find(|x| x.id == pkg.id)
					.expect("Project should have been fetched");
				// Get the team associated with this project. Teams can have no members, which we handle by just using an empty team
				let empty_vec = Vec::new();
				let team = modrinth_teams
					.iter()
					.find(|team| team.iter().any(|member| member.team_id == project.team))
					.unwrap_or(&empty_vec);
				super::modrinth::gen_raw(
					project.clone(),
					&modrinth_versions,
					team,
					pkg.config.relation_substitutions,
					&pkg.config.force_extensions,
				)
				.await
			}
		};

		// Merge with config
		let mut package =
			serde_json::value::to_value(package).expect("Failed to convert package to value");
		let merge = serde_json::value::to_value(pkg.config.merge)
			.expect("Failed to convert merged config to value");
		json_merge(&mut package, merge);

		// Write out the package
		let path = PathBuf::from(&config.output_dir)
			.join(format!("{}.json", pkg.pkg_id.expect("Package ID missing")));
		let file =
			BufWriter::new(File::create(path).expect("Failed to create package output file"));
		serde_json::to_writer_pretty(file, &package).expect("Failed to write package to file");
	}
}
