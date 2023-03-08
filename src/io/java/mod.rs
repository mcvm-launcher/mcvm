pub mod args;
pub mod classpath;

use crate::io::files::{self, paths::Paths};
use crate::net::download::{Download, DownloadError};
use crate::util::json::{self, JsonType};
use crate::util::mojang::{ARCH_STRING, OS_STRING};
use crate::util::print::ReplPrinter;

use color_print::cformat;
use libflate::gzip::Decoder;
use tar::Archive;

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum JavaKind {
	Adoptium(Option<String>),
	Custom(PathBuf),
}

impl JavaKind {
	pub fn from_str(string: &str) -> Self {
		match string {
			"adoptium" => Self::Adoptium(None),
			path => Self::Custom(PathBuf::from(String::from(shellexpand::tilde(path)))),
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum JavaError {
	#[error("File operation failed:\n{}", .0)]
	Io(#[from] std::io::Error),
	#[error("Failed to download file:\n{}", .0)]
	Download(#[from] DownloadError),
	#[error("Failed to parse json file:\n{}", .0)]
	Json(#[from] json::JsonError),
	#[error("No valid installation was found for your system")]
	InstallationNotFound,
}

#[derive(Debug)]
pub struct Java {
	kind: JavaKind,
	pub path: Option<PathBuf>,
}

impl Java {
	pub fn new(kind: JavaKind) -> Self {
		Self { kind, path: None }
	}

	pub fn install(&mut self, paths: &Paths, verbose: bool, force: bool) -> Result<(), JavaError> {
		match &self.kind {
			JavaKind::Adoptium(major_version) => {
				let major_version = major_version.as_ref().expect("Major version should exist");
				let mut printer = ReplPrinter::new(verbose);

				let out_dir = paths.java.join("adoptium");
				files::create_dir(&out_dir)?;
				let url = format!(
					"https://api.adoptium.net/v3/assets/latest/{}/hotspot?image_type=jre&vendor=eclipse&architecture={}&os={}",
					major_version,
					ARCH_STRING,
					OS_STRING
				);
				let mut dwn = Download::new();
				dwn.url(&url)?;
				dwn.follow_redirects()?;
				dwn.add_str();
				dwn.perform()?;

				let manifest_val = json::parse_json(&dwn.get_str()?)?;
				let manifest = json::ensure_type(manifest_val.as_array(), JsonType::Arr)?;
				let version = json::ensure_type(
					manifest
						.get(0)
						.ok_or(JavaError::InstallationNotFound)?
						.as_object(),
					JsonType::Obj,
				)?;
				let bin_url = json::access_str(
					json::access_object(json::access_object(version, "binary")?, "package")?,
					"link",
				)?;
				let mut extracted_bin_name = json::access_str(version, "release_name")?.to_string();
				extracted_bin_name.push_str("-jre");
				let extracted_bin_dir = out_dir.join(&extracted_bin_name);

				self.path = Some(extracted_bin_dir.clone());
				if !force && extracted_bin_dir.exists() {
					return Ok(());
				}

				let tar_name = "adoptium".to_owned() + &major_version + ".tar.gz";
				let tar_path = out_dir.join(tar_name);

				dwn.reset();
				dwn.url(bin_url)?;
				dwn.follow_redirects()?;
				dwn.add_file(&tar_path)?;
				printer.print(&cformat!(
					"\tDownloading Adoptium Temurin JRE <b>{}</b>...",
					json::access_str(version, "release_name")?
				));
				dwn.perform()?;
				// Close the files
				dwn.reset();

				// Extraction
				printer.print(&cformat!("\tExtracting..."));
				let data = fs::read(&tar_path)?;
				let mut decoder = Decoder::new(data.as_slice())?;
				let mut arc = Archive::new(&mut decoder);
				arc.unpack(out_dir)?;
				printer.print(&cformat!("\t<g>Java installation finished."));
				Ok(())
			}
			JavaKind::Custom(path) => {
				self.path = Some(path.clone());

				Ok(())
			}
		}
	}
}
