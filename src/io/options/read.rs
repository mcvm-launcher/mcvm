use std::io::Read;

use anyhow::Context;
use serde::Deserialize;

use crate::util::ToInt;

use super::{client::ClientOptions, server::ServerOptions};

/// General options structure used to produce options for both client and server
#[derive(Deserialize, Debug, Clone)]
pub struct Options {
	#[serde(default)]
	pub client: ClientOptions,
	#[serde(default)]
	pub server: ServerOptions,
}

/// Used for values that can be string representations or custom numbers
#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum OptionsEnum<T: Clone + ToInt> {
	Mode(T),
	Number(i32),
}

impl<T: Clone + ToInt> ToInt for OptionsEnum<T> {
	fn to_int(&self) -> i32 {
		match self {
			Self::Mode(mode) => mode.to_int(),
			Self::Number(num) => *num,
		}
	}
}

pub fn parse_options<R: Read>(reader: &mut R) -> anyhow::Result<Options> {
	serde_json::from_reader(reader).context("Failed to parse options")
}

#[cfg(test)]
pub fn parse_options_str(string: &str) -> anyhow::Result<Options> {
	serde_json::from_str(string).context("Failed to parse options")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default() {
		parse_options_str("{}").unwrap();
	}
}
