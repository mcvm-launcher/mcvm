use oauth2::ClientId;

use crate::util::secrets::get_ms_client_id;

/// Configuration for different parts of the core library
pub struct Configuration {
	/// The Microsoft client ID to use for Microsoft authentication
	pub(crate) ms_client_id: ClientId,
}

impl Default for Configuration {
	fn default() -> Self {
		Self::new()
	}
}

impl Configuration {
	/// Construct the default configuration
	pub fn new() -> Self {
		Self {
			ms_client_id: get_ms_client_id().into(),
		}
	}

	/// Get a builder for the configuration
	pub fn builder() -> ConfigBuilder {
		ConfigBuilder::new()
	}
}

/// Simple builder for the configuration
pub struct ConfigBuilder {
	config: Configuration,
}

impl ConfigBuilder {
	/// Start a new ConfigBuilder with default configuration
	pub fn new() -> Self {
		Self {
			config: Configuration::new(),
		}
	}

	/// Finish building and get the configuration
	pub fn build(self) -> Configuration {
		self.config
	}

	/// Set the Microsoft client ID to use for Microsoft / XBox Live authentication
	pub fn ms_client_id(mut self, ms_client_id: ClientId) -> Self {
		self.config.ms_client_id = ms_client_id;
		self
	}
}
