mod smithed_api;

use std::collections::HashMap;
use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use mcvm::data::config::ConfigDeser;
use mcvm::options::Options;
use mcvm::pkg_crate::declarative::{
	DeclarativeAddon, DeclarativeAddonVersion, DeclarativeConditionSet, DeclarativePackageRelations,
};
use mcvm::pkg_crate::metadata::PackageMetadata;
use mcvm::pkg_crate::properties::PackageProperties;
use mcvm::pkg_crate::{declarative::DeclarativePackage, repo::RepoPkgIndex};
use mcvm::shared::addon::AddonKind;
use mcvm::shared::util::DeserListOrSingle;
use mcvm::shared::versions::VersionPattern;

#[tokio::main]
async fn main() {
	let cli = Cli::parse();
	match cli.command {
		Subcommand::Schemas => gen_schemas(),
		Subcommand::SmithedPkg {
			id,
			dep_substitutions,
		} => gen_smithed_pkg(&id, dep_substitutions).await,
	}
}

#[derive(clap::Parser)]
struct Cli {
	#[command(subcommand)]
	command: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
	Schemas,
	SmithedPkg {
		#[arg(short, long)]
		dep_substitutions: Option<Vec<String>>,
		id: String,
	},
}

fn gen_schemas() {
	let dir = PathBuf::from("./schemas");
	if !dir.exists() {
		std::fs::create_dir(&dir).expect("Failed to create schema directory");
	}
	// I would seriously recommend adding schemars.schema_for to your rust-analyzer
	// proc-macro ignore list
	let schemas = [
		(
			schemars::schema_for!(DeclarativePackage),
			"declarative.json",
		),
		(schemars::schema_for!(RepoPkgIndex), "pkg_repo.json"),
		(schemars::schema_for!(Options), "options.json"),
		(schemars::schema_for!(ConfigDeser), "config.json"),
	];
	for (schema, filename) in schemas {
		let file = File::create(dir.join(filename)).expect("Failed to create schema file");
		let mut file = BufWriter::new(file);
		serde_json::to_writer_pretty(&mut file, &schema).expect("Failed to write schema to file");
	}
}

async fn gen_smithed_pkg(id: &str, dep_substitutions: Option<Vec<String>>) {
	let mut dep_subs = HashMap::new();
	if let Some(dep_substitutions) = dep_substitutions {
		for dep in dep_substitutions {
			let mut items = dep.split('=');
			let key = items.next().expect("Key in dep sub is missing");
			let val = items.next().expect("Val in dep sub is missing");
			if key.is_empty() {
				panic!("Dep sub key is empty");
			}
			if val.is_empty() {
				panic!("Dep sub value is empty");
			}
			dep_subs.insert(key.to_string(), val.to_string());
		}
	}

	let pack = smithed_api::get_pack(id).await.expect("Failed to get pack");

	let meta = PackageMetadata {
		name: Some(pack.display.name),
		description: Some(pack.display.description),
		icon: Some(pack.display.icon),
		website: pack.display.web_page,
		..Default::default()
	};

	let props = PackageProperties {
		smithed_id: Some(pack.id),
		tags: Some(vec!["datapack".into()]),
		..Default::default()
	};

	// Generate addons

	let mut datapack = DeclarativeAddon {
		kind: AddonKind::Datapack,
		versions: Vec::new(),
		conditions: Vec::new(),
	};

	let mut resourcepack = DeclarativeAddon {
		kind: AddonKind::ResourcePack,
		versions: Vec::new(),
		conditions: Vec::new(),
	};

	for version in pack.versions {
		let version_name_sanitized = version.name.replace('.', "-");
		let version_name = format!("smithed-version-{version_name_sanitized}");
		let mc_versions: Vec<VersionPattern> = version
			.supports
			.iter()
			.map(|x| VersionPattern::Single(x.clone()))
			.collect();

		let deps: Vec<String> = version
			.dependencies
			.iter()
			.map(|dep| {
				if let Some(dep_id) = dep_subs.get(&dep.id) {
					dep_id.clone()
				} else {
					panic!("Dependency {} was not substituted", dep.id)
				}
			})
			.collect();

		let mut pkg_version = DeclarativeAddonVersion {
			version: Some(version_name),
			conditional_properties: DeclarativeConditionSet {
				minecraft_versions: Some(DeserListOrSingle::List(mc_versions)),
				..Default::default()
			},
			relations: DeclarativePackageRelations {
				dependencies: DeserListOrSingle::List(deps),
				..Default::default()
			},
			..Default::default()
		};

		if let Some(url) = version.downloads.datapack {
			pkg_version.url = Some(url);
			datapack.versions.push(pkg_version.clone());
		}

		if let Some(url) = version.downloads.resourcepack {
			pkg_version.url = Some(url);
			resourcepack.versions.push(pkg_version.clone());
		}
	}

	let mut addon_map = HashMap::new();
	addon_map.insert("datapack".into(), datapack);
	addon_map.insert("resourcepack".into(), resourcepack);

	let pkg = DeclarativePackage {
		meta,
		properties: props,
		addons: addon_map,
		..Default::default()
	};

	println!(
		"{}",
		serde_json::to_string_pretty(&pkg).expect("Failed to format package")
	);
}
