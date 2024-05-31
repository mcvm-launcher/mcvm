use std::{fs::File, io::BufWriter, path::PathBuf};

use clap::Parser;
use mcvm::data::config::ConfigDeser;
use mcvm::pkg_crate::{declarative::DeclarativePackage, repo::RepoIndex};
use mcvm_options::Options;

#[tokio::main]
async fn main() {
	let cli = Cli::parse();
	match cli.command {
		Subcommand::Schemas => gen_schemas(),
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
