use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use anyhow::Context;
use mcvm_core::io::{json_from_file, json_to_file};
use mcvm_shared::{pkg::PackageSearchParameters, util::utc_timestamp};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A cache for search results in a custom plugin repository that holds entries for a certain amount of time
pub struct PackageSearchCache {
	max_age: u64,
	contents: CacheContents,
	path: PathBuf,
}

impl PackageSearchCache {
	/// Opens the cache at the given JSON file given the max age for an entry in seconds
	pub fn open(path: impl AsRef<Path>, max_age: u64) -> anyhow::Result<Self> {
		let contents = if path.as_ref().exists() {
			json_from_file(path.as_ref()).context("Failed to read cache from file")?
		} else {
			let default = CacheContents::default();
			let _ = json_to_file(path.as_ref(), &default);
			default
		};

		Ok(Self {
			max_age,
			contents,
			path: path.as_ref().to_path_buf(),
		})
	}

	/// Checks the cache for cached results
	pub fn check<D: DeserializeOwned>(&self, search: &PackageSearchParameters) -> Option<D> {
		// Don't cache searches with queries as users will want up to date results
		if search.search.is_some() {
			return None;
		}

		let search = serde_json::to_string(search).ok()?;
		let entry = self.contents.entries.get(&search)?;

		// Invalidate the entry if it is too old
		let timestamp = utc_timestamp().ok()?;
		if timestamp - entry.timestamp > self.max_age {
			return None;
		} else {
			let results = serde_json::from_value(entry.results.clone()).ok()?;
			return Some(results);
		}
	}

	/// Writes to the cache
	pub fn write<S: Serialize>(
		&mut self,
		search: &PackageSearchParameters,
		results: S,
	) -> anyhow::Result<()> {
		let search = serde_json::to_string(search).context("Failed to stringify search")?;
		let results = serde_json::to_value(results).context("Failed to serialize results")?;
		let timestamp = utc_timestamp().context("Failed to get UTC timestamp")?;

		self.contents
			.entries
			.insert(search, CacheEntry { timestamp, results });

		json_to_file(&self.path, &self.contents).context("Failed to write to cache")
	}
}

#[derive(Serialize, Deserialize, Default)]
struct CacheContents {
	entries: HashMap<String, CacheEntry>,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
	timestamp: u64,
	results: serde_json::Value,
}
