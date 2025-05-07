use std::fs::File;

use zip::{write::FileOptions, CompressionMethod, ZipWriter};

macro_rules! add_file {
	($zip:expr, $path:literal) => {
		let path = concat!("../docs/docs/", $path);
		$zip.start_file(
			$path,
			FileOptions::<()>::default().compression_method(CompressionMethod::Deflated),
		)
		.unwrap();
		std::io::copy(&mut File::open(path).unwrap(), &mut $zip).unwrap();
		println!("cargo::rerun-if-changed={path}");
	};
}

fn main() {
	let out = File::create("./zipped_docs.zip").unwrap();
	let mut zip = ZipWriter::new(out);

	add_file!(zip, "index.md");
	add_file!(zip, "configuring.md");
	add_file!(zip, "modifications.md");
	add_file!(zip, "principles.md");
	zip.add_directory("packages", FileOptions::<()>::default())
		.unwrap();
	add_file!(zip, "packages/declarative.md");
	add_file!(zip, "packages/index.md");
	add_file!(zip, "packages/scripts.md");
	zip.add_directory("guide", FileOptions::<()>::default())
		.unwrap();
	add_file!(zip, "guide/index.md");
	add_file!(zip, "guide/packages.md");
	zip.add_directory("plugins", FileOptions::<()>::default())
		.unwrap();
	add_file!(zip, "plugins/index.md");
	add_file!(zip, "plugins/user_guide.md");
	zip.add_directory("plugins/development", FileOptions::<()>::default())
		.unwrap();
	add_file!(zip, "plugins/development/index.md");
	add_file!(zip, "plugins/development/format.md");
	add_file!(zip, "plugins/development/hooks.md");
	add_file!(zip, "plugins/plugins/index.md");
	add_file!(zip, "plugins/plugins/args.md");
	add_file!(zip, "plugins/plugins/backup.md");
	add_file!(zip, "plugins/plugins/custom_files.md");
	add_file!(zip, "plugins/plugins/docs.md");
	add_file!(zip, "plugins/plugins/extra_versions.md");
	add_file!(zip, "plugins/plugins/gen_pkg.md");
	add_file!(zip, "plugins/plugins/lang.md");
	add_file!(zip, "plugins/plugins/modrinth_api.md");
	add_file!(zip, "plugins/plugins/options.md");
	add_file!(zip, "plugins/plugins/scripthook.md");
	add_file!(zip, "plugins/plugins/server_restart.md");
	add_file!(zip, "plugins/plugins/stats.md");
	add_file!(zip, "plugins/plugins/config_split.md");
	add_file!(zip, "plugins/plugins/smithed_api.md");
	add_file!(zip, "plugins/plugins/smithed.md");
	add_file!(zip, "plugins/plugins/modrinth.md");
}
