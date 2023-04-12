pub mod args;
pub mod classpath;

use crate::data::profile::update::UpdateManager;
use crate::io::files::{self, paths::Paths};
use crate::net;
use crate::net::download::download_file;
use crate::util::json;
use crate::util::print::ReplPrinter;

use anyhow::Context;
use color_print::cformat;
use libflate::gzip::Decoder;
use tar::Archive;

use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};

use super::Later;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum JavaKind {
	Adoptium(Later<String>),
	Custom(PathBuf),
}

impl JavaKind {
	pub fn from_str(string: &str) -> Self {
		match string {
			"adoptium" => Self::Adoptium(Later::Empty),
			path => Self::Custom(PathBuf::from(String::from(shellexpand::tilde(path)))),
		}
	}
}

/// A Java installation used to launch the game
#[derive(Debug, Clone)]
pub struct Java {
	kind: JavaKind,
	pub path: Later<PathBuf>,
}

impl Java {
	pub fn new(kind: JavaKind) -> Self {
		Self {
			kind,
			path: Later::Empty,
		}
	}

	/// Add a major version to a Java installation that supports it
	pub fn add_version(&mut self, version: &str) {
		match &mut self.kind {
			JavaKind::Adoptium(vers) => vers.fill(version.to_owned()),
			JavaKind::Custom(..) => {}
		};
	}

	/// Download / install all needed files
	pub async fn install(
		&mut self,
		paths: &Paths,
		manager: &UpdateManager,
	) -> anyhow::Result<HashSet<PathBuf>> {
		let mut out = HashSet::new();
		match &self.kind {
			JavaKind::Adoptium(major_version) => {
				let mut printer = ReplPrinter::from_options(manager.print.clone());

				let out_dir = paths.java.join("adoptium");
				files::create_dir(&out_dir)?;
				let version = net::java::adoptium::get_latest(major_version.get())
					.await
					.context("Failed to obtain Adoptium information")?;
				let bin_url = json::access_str(
					json::access_object(json::access_object(&version, "binary")?, "package")?,
					"link",
				)?;
				let mut extracted_bin_name =
					json::access_str(&version, "release_name")?.to_string();
				extracted_bin_name.push_str("-jre");
				let extracted_bin_dir = out_dir.join(&extracted_bin_name);

				self.path.fill(extracted_bin_dir.clone());
				if !manager.should_update_file(&extracted_bin_dir) {
					return Ok(out);
				}
				out.insert(extracted_bin_dir.clone());

				let arc_extension = if cfg!(windows) { ".zip" } else { ".tar.gz" };
				let arc_name = format!("adoptium{}{arc_extension}", major_version.get());
				let arc_path = out_dir.join(arc_name);

				printer.print(&cformat!(
					"Downloading Adoptium Temurin JRE <b>{}</b>...",
					json::access_str(&version, "release_name")?
				));
				download_file(bin_url, &arc_path)
					.await
					.context("Failed to download JRE binaries")?;

				// Extraction
				printer.print(&cformat!("Extracting JRE..."));
				extract_adoptium_archive(&arc_path, &out_dir).context("Failed to extract")?;
				printer.print(&cformat!("Removing archive..."));
				tokio::fs::remove_file(arc_path)
					.await
					.context("Failed to remove archive")?;
				printer.print(&cformat!("<g>Java installation finished."));
			}
			JavaKind::Custom(path) => {
				self.path.fill(path.clone());
			}
		}
		Ok(out)
	}
}

/// Extracts the Adoptium JRE archive (either a tar or a zip)
fn extract_adoptium_archive(arc_path: &Path, out_dir: &Path) -> anyhow::Result<()> {
	let mut file = File::open(arc_path).context("Failed to read archive file")?;
	if cfg!(windows) {
		zip_extract::extract(&mut file, out_dir, false).context("Failed to extract zip file")?;
	} else {
		let mut decoder = Decoder::new(&mut file).context("Failed to decode tar.gz")?;
		let mut arc = Archive::new(&mut decoder);
		arc.unpack(out_dir).context("Failed to unarchive tar")?;
	}

	Ok(())
}
