use crate::package::repo::PkgRepo;

use serde::Deserialize;

#[derive(Debug)]
pub struct ConfigPreferences {
	pub repositories: Vec<PkgRepo>
}

struct PrefSerialize {
	
}

impl ConfigPreferences {
	pub fn new() -> Self {
		Self {
			repositories: vec![]
		}
	}
}
