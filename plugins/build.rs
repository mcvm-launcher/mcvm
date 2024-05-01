use std::fs::File;

use zip::{write::FileOptions, CompressionMethod, ZipWriter};

macro_rules! add_file {
	($zip:expr, $path:literal) => {
		let path = concat!("../docs/", $path);
		$zip.start_file(
			$path,
			FileOptions::default().compression_method(CompressionMethod::Deflated),
		)
		.unwrap();
		std::io::copy(&mut File::open(path).unwrap(), &mut $zip).unwrap();
		println!("cargo::rerun-if-changed={path}");
	};
}

fn main() {
	let out = File::create("./zipped_docs.zip").unwrap();
	let mut zip = ZipWriter::new(out);

	add_file!(zip, "README.md");
	add_file!(zip, "configuring.md");
	add_file!(zip, "game_options.md");
	add_file!(zip, "modifications.md");
	add_file!(zip, "principles.md");
	zip.add_directory("packages", FileOptions::default())
		.unwrap();
	add_file!(zip, "packages/declarative.md");
	add_file!(zip, "packages/packages.md");
	add_file!(zip, "packages/scripts.md");
}
