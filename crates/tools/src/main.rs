mod gen_pkg;
mod smithed_api;

use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use gen_pkg::PackageSource;
use mcvm::data::config::ConfigDeser;
use mcvm::options::Options;
use mcvm::pkg_crate::{declarative::DeclarativePackage, repo::RepoIndex};

#[tokio::main]
async fn main() {
	let cli = Cli::parse();
	match cli.command {
		Subcommand::Schemas => gen_schemas(),
		Subcommand::GenPkg {
			config_path,
			source,
			id,
		} => {
			let config = config_path.map(|config_path| {
				serde_json::from_reader(
					File::open(config_path).expect("Failed to open config file"),
				)
				.expect("Failed to deserialize config")
			});
			gen_pkg::gen(source, config, &id).await;
		}
		Subcommand::GenPkgBatched { config_path } => {
			let config = serde_json::from_reader(
				File::open(config_path).expect("Failed to open config file"),
			)
			.expect("Failed to deserialize config");
			gen_pkg::batched::batched_gen(config).await;
		}
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
	GenPkg {
		/// Path to configuration for the package generation
		#[arg(short, long)]
		config_path: Option<String>,
		/// The source to get the package from
		source: PackageSource,
		/// The ID of the package from whatever source it is from
		id: String,
	},
	GenPkgBatched {
		/// Path to configuration for the package generation
		config_path: String,
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
		(schemars::schema_for!(RepoIndex), "pkg_repo.json"),
		(schemars::schema_for!(Options), "options.json"),
		(schemars::schema_for!(ConfigDeser), "config.json"),
	];
	for (schema, filename) in schemas {
		let file = File::create(dir.join(filename)).expect("Failed to create schema file");
		let mut file = BufWriter::new(file);
		serde_json::to_writer_pretty(&mut file, &schema).expect("Failed to write schema to file");
	}
}
