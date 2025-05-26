use std::collections::HashMap;
use std::fs::File;
use std::io::stdout;

use clap::Parser;
use mcvm_pkg_gen::{modrinth, smithed};
use mcvm_plugin::api::CustomPlugin;
use serde::{Deserialize, Serialize};
use serde_json::ser::PrettyFormatter;
use serde_json::Serializer;

/// Generation of many packages
pub mod batched;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("gen_pkg", include_str!("plugin.json"))?;
	plugin.subcommand(|_, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		let subcommand = subcommand.to_owned();

		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));

		let runtime = tokio::runtime::Runtime::new()?;
		runtime.block_on(async move {
			if subcommand == "gen-pkg" {
				let cli = GenPkg::parse_from(it);
				let config = cli.config_path.map(|config_path| {
					serde_json::from_reader(
						File::open(config_path).expect("Failed to open config file"),
					)
					.expect("Failed to deserialize config")
				});
				gen(cli.source, config, &cli.id).await;
			} else if subcommand == "gen-pkg-batched" {
				let cli = GenPkgBatched::parse_from(it);
				let config = serde_json::from_reader(
					File::open(cli.config_path).expect("Failed to open config file"),
				)
				.expect("Failed to deserialize config");
				batched::batched_gen(config, cli.filter).await;
			}

			Ok::<(), anyhow::Error>(())
		})?;

		Ok(())
	})?;

	Ok(())
}

#[derive(Parser)]
struct GenPkg {
	/// Path to configuration for the package generation
	#[arg(short, long)]
	config_path: Option<String>,
	/// The source to get the package from
	source: PackageSource,
	/// The ID of the package from whatever source it is from
	id: String,
}

#[derive(Parser)]
struct GenPkgBatched {
	/// Path to configuration for the package generation
	config_path: String,
	/// Packages to filter and only include
	#[arg(short, long)]
	filter: Vec<String>,
}

/// Different types of package generation
#[derive(Copy, Clone, Debug, clap::ValueEnum, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageSource {
	Smithed,
	Modrinth,
}

/// Configuration for generating the package from whatever source
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct PackageGenerationConfig {
	/// JSON structure to be merged with the output package
	pub merge: serde_json::Value,
	/// Substitutions for relations
	pub relation_substitutions: HashMap<String, String>,
	/// Dependencies to force into extensions
	pub force_extensions: Vec<String>,
	/// Whether to make fabric modloaders fabriclike instead
	pub make_fabriclike: Option<bool>,
	/// Whether to make forge modloaders forgelike instead
	pub make_forgelike: Option<bool>,
}

impl PackageGenerationConfig {
	/// Merge this config with another one to be placed over top of it
	#[must_use]
	pub fn merge(mut self, other: Self) -> Self {
		mcvm_core::util::json::merge(&mut self.merge, other.merge);
		self.relation_substitutions
			.extend(other.relation_substitutions);
		self.force_extensions.extend(other.force_extensions);
		self.make_fabriclike = other.make_fabriclike.or(self.make_fabriclike);
		self.make_forgelike = other.make_forgelike.or(self.make_forgelike);

		self
	}
}

/// Generates a package from a source and config
pub async fn gen(source: PackageSource, config: Option<PackageGenerationConfig>, id: &str) {
	let config = config.unwrap_or_default();
	let mut pkg = match source {
		PackageSource::Smithed => {
			smithed::gen_from_id(
				id,
				config.relation_substitutions,
				&config.force_extensions,
				true,
			)
			.await
		}
		PackageSource::Modrinth => {
			modrinth::gen_from_id(
				id,
				config.relation_substitutions,
				&config.force_extensions,
				config.make_fabriclike.unwrap_or_default(),
				config.make_forgelike.unwrap_or_default(),
			)
			.await
		}
	};

	// Improve the generated package
	pkg.improve_generation();
	pkg.optimize();

	// Merge with config
	let mut pkg = serde_json::value::to_value(pkg).expect("Failed to convert package to value");
	mcvm_core::util::json::merge(&mut pkg, config.merge);

	let mut serializer = Serializer::with_formatter(stdout(), PrettyFormatter::with_indent(b"\t"));
	pkg.serialize(&mut serializer)
		.expect("Failed to output package");
}
