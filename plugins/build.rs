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
	add_file!(zip, "Configuring.md");
	add_file!(zip, "Game Options.md");
	add_file!(zip, "Modifications.md");
	add_file!(zip, "Principles.md");
	zip.add_directory("Packages", FileOptions::default())
		.unwrap();
	add_file!(zip, "Packages/Declarative.md");
	add_file!(zip, "Packages/Packages.md");
	add_file!(zip, "Packages/Scripts.md");
}
