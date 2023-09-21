use std::{fs::File, io::BufWriter, path::PathBuf};

use mcvm::{
	data::config::ConfigDeser,
	io::options::Options,
	pkg_crate::{declarative::DeclarativePackage, repo::RepoPkgIndex},
};

fn main() {
	let args = std::env::args();
	let dir = PathBuf::from("./schemas");
	if !dir.exists() {
		std::fs::create_dir(&dir).expect("Failed to create schema directory");
	}
	// I would seriously recommend adding schemars.schema_for to your rust-analyzer
	// proc-macro ignore list
	for arg in args {
		let (schema, filename) = match arg.as_str() {
			"declarative" => {
				let schema = schemars::schema_for!(DeclarativePackage);
				(schema, "declarative.json")
			}
			"pkg_repo" => {
				let schema = schemars::schema_for!(RepoPkgIndex);
				(schema, "pkg_repo.json")
			}
			"options" => {
				let schema = schemars::schema_for!(Options);
				(schema, "options.json")
			}
			"config" => {
				let schema = schemars::schema_for!(ConfigDeser);
				(schema, "config.json")
			}
			other => {
				println!("Unknown schema type '{other}'");
				continue;
			}
		};
		let file = File::create(dir.join(filename)).expect("Failed to create schema file");
		let mut file = BufWriter::new(file);
		serde_json::to_writer_pretty(&mut file, &schema).expect("Failed to write schema to file");
	}
}
