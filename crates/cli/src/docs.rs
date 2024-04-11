use std::{collections::HashMap, io::Cursor};

use anyhow::Context;
use zip::ZipArchive;

static DOCS: &[u8] = include_bytes!("../zipped_docs.zip");

/// Manager struct for the stored docs
pub struct Docs {
	docs: HashMap<String, String>,
}

impl Docs {
	/// Load the docs
	pub fn load() -> anyhow::Result<Self> {
		let mut zip = ZipArchive::new(Cursor::new(DOCS))
			.context("Failed to open compressed documentation")?;
		let mut docs = HashMap::new();
		for i in 0..zip.len() {
			let mut file = zip.by_index(i).expect("Should exist");
			if file.is_dir() {
				continue;
			}

			let mut doc = Cursor::new(Vec::new());
			std::io::copy(&mut file, &mut doc).context("Failed to copy zipped doc")?;
			let doc = String::from_utf8(doc.into_inner())
				.context("Documentation was not in valid UTF-8")?;

			docs.insert(file.name().to_string(), doc);
		}

		Ok(Self { docs })
	}

	/// Get one of the documentation pages by name
	pub fn get_page(&self, doc: &str) -> Option<&String> {
		self.docs.get(doc)
	}

	/// Get the list of doc pages
	pub fn get_pages(&self) -> Vec<String> {
		self.docs.keys().cloned().collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_docs_loading() {
		Docs::load().unwrap();
	}
}
