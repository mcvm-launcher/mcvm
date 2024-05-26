use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;
use std::{cmp::Reverse, collections::HashMap};

use iso8601_timestamp::Timestamp;
use mcvm::net::modrinth::Version;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{ser::PrettyFormatter, Serializer};
use tokio::sync::Mutex;
use tokio::task::JoinSet;

use crate::gen_pkg::json_merge;
use crate::smithed_api;

use super::{PackageGenerationConfig, PackageSource};

/// Configuration for a lot of package generation
#[derive(Deserialize)]
pub struct BatchedConfig {
	/// The packages to generate
	#[serde(default)]
	pub packages: Vec<BatchedPackageConfig>,
	/// The output directory for the packages
	pub output_dir: String,
	/// A directory to read packages from
	#[serde(default)]
	pub config_dir: Option<String>,
	/// Global package config to apply to all packages
	#[serde(default)]
	pub global_config: Option<PackageGenerationConfig>,
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
pub async fn batched_gen(mut config: BatchedConfig, filter: Vec<String>) {
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
		.filter(|x| {
			if !filter.is_empty()
				&& !filter.contains(x.pkg_id.as_ref().expect("Package ID should exist"))
			{
				return false;
			}
			x.source == PackageSource::Modrinth
		})
		.map(|x| x.id.clone())
		.collect();
	let modrinth_projects = mcvm::net::modrinth::get_multiple_projects(&modrinth_ids, &client)
		.await
		.expect("Failed to get Modrinth projects");

	// Collect Modrinth project versions. We have to batch these into multiple requests because there becomes
	// just too many parameters for the URL to handle
	let batch_limit = 215;
	let modrinth_version_ids: Vec<_> = modrinth_projects
		.iter()
		.flat_map(|x| x.versions.iter().cloned())
		.collect();
	if !modrinth_version_ids.is_empty() {
		println!(
			"Downloading {} Modrinth versions...",
			modrinth_version_ids.len()
		);
	}

	let chunks = modrinth_version_ids.chunks(batch_limit);

	let modrinth_versions = Arc::new(Mutex::new(Vec::new()));
	let mut tasks = JoinSet::new();
	for chunk in chunks {
		let chunk = chunk.to_vec();
		let client = client.clone();
		let modrinth_versions = modrinth_versions.clone();
		let task = async move {
			let versions = mcvm::net::modrinth::get_multiple_versions(&chunk, &client)
				.await
				.expect("Failed to get Modrinth versions");
			let mut lock = modrinth_versions.lock().await;
			lock.extend(versions);
		};
		tasks.spawn(task);
	}

	// Download Smithed packs at the same time
	let smithed_packs = Arc::new(Mutex::new(Vec::new()));
	for pkg in &config.packages {
		if let PackageSource::Smithed = pkg.source {
			let client = client.clone();
			let smithed_packs = smithed_packs.clone();
			let id = pkg.id.clone();
			let task = async move {
				let pack = smithed_api::get_pack(&id, &client)
					.await
					.expect("Failed to get Smithed pack");
				let mut lock = smithed_packs.lock().await;
				lock.push(pack);
			};
			tasks.spawn(task);
		}
	}

	// Download Modrinth teams at the same time
	let mut modrinth_team_ids = Vec::new();
	for project in &modrinth_projects {
		modrinth_team_ids.push(project.team.clone());
	}
	let modrinth_teams = Arc::new(Mutex::new(Vec::new()));
	{
		let client = client.clone();
		let modrinth_teams = modrinth_teams.clone();
		let task = async move {
			let teams = mcvm::net::modrinth::get_multiple_teams(&modrinth_team_ids, &client)
				.await
				.expect("Failed to get Modrinth teams");
			let mut lock = modrinth_teams.lock().await;
			*lock = teams;
		};
		tasks.spawn(task);
	}

	// Run the tasks
	while let Some(result) = tasks.join_next().await {
		result.expect("Task failed");
	}
	let mut modrinth_versions = modrinth_versions.lock().await;
	let smithed_packs = smithed_packs.lock().await;
	let modrinth_teams = modrinth_teams.lock().await;

	// Sort the Modrinth versions
	modrinth_versions.sort_by_key(SortVersions::new);

	// Put the Modrinth versions into a HashMap for better performance
	let mut modrinth_version_map: HashMap<_, Vec<_>> = HashMap::new();
	for version in modrinth_versions.iter() {
		let entry = modrinth_version_map
			.entry(version.project_id.clone())
			.or_default();

		entry.push(version.clone());
	}

	// Iterate through the packages to generate
	println!("Generating packages...");
	for pkg in config.packages {
		let pkg_id = pkg.pkg_id.as_ref().expect("Package ID should exist");
		if !filter.is_empty() && !filter.contains(pkg_id) {
			continue;
		}

		println!("Generating package {}", pkg_id);
		let pkg_config = if let Some(global_config) = &config.global_config {
			global_config.clone().merge(pkg.config)
		} else {
			pkg.config
		};

		let mut package = match pkg.source {
			PackageSource::Smithed => {
				let pack = smithed_packs
					.iter()
					.find(|x| x.id == pkg.id)
					.expect("Smithed pack should have been downloaded");
				super::smithed::gen_raw(
					pack.clone(),
					pkg_config.relation_substitutions,
					&pkg_config.force_extensions,
				)
				.await
			}
			PackageSource::Modrinth => {
				// Get the project
				let project = modrinth_projects
					.iter()
					.find(|x| x.id == pkg.id)
					.expect("Project should have been fetched");

				// Get the versions for the project
				let versions = modrinth_version_map
					.get(&pkg.id)
					.expect("Project versions missing from map");

				// Get the team associated with this project. Teams can have no members, which we handle by just using an empty team
				let empty_vec = Vec::new();
				let team = modrinth_teams
					.iter()
					.find(|team| team.iter().any(|member| member.team_id == project.team))
					.unwrap_or(&empty_vec);

				super::modrinth::gen_raw(
					project.clone(),
					versions,
					team,
					pkg_config.relation_substitutions,
					&pkg_config.force_extensions,
					pkg_config.make_fabriclike.unwrap_or_default(),
					pkg_config.make_forgelike.unwrap_or_default(),
				)
				.await
			}
		};

		// Improve the generated package
		package.improve_generation();
		package.optimize();

		// Merge with config
		let mut package =
			serde_json::value::to_value(package).expect("Failed to convert package to value");
		let merge = serde_json::value::to_value(pkg_config.merge)
			.expect("Failed to convert merged config to value");
		json_merge(&mut package, merge);

		// Write out the package
		let path = PathBuf::from(&config.output_dir)
			.join(format!("{}.json", pkg.pkg_id.expect("Package ID missing")));
		let file =
			BufWriter::new(File::create(path).expect("Failed to create package output file"));

		let mut serializer = Serializer::with_formatter(file, PrettyFormatter::with_indent(b"\t"));
		package
			.serialize(&mut serializer)
			.expect("Failed to serialize JSON");
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct SortVersions {
	featured: Reverse<bool>,
	timestamp: Reverse<Timestamp>,
}

impl SortVersions {
	fn new(version: &Version) -> Self {
		Self {
			featured: Reverse(version.featured),
			timestamp: Reverse(
				Timestamp::parse(&version.date_published).unwrap_or(Timestamp::UNIX_EPOCH),
			),
		}
	}
}
