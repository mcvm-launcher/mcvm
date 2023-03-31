use std::{io::Read, fmt::Display};

use anyhow::Context;
use serde::Deserialize;

use crate::util::ToInt;

use super::Options;

// /// Used for values that can be string representations or custom numbers
// #[derive(Deserialize, PartialEq, Debug, Clone)]
// #[serde(untagged)]
// pub enum EnumOrNumber<T: Clone + ToInt> {
// 	Mode(T),
// 	Number(i32),
// }

// impl<T: Clone + ToInt> ToInt for EnumOrNumber<T> {
// 	fn to_int(&self) -> i32 {
// 		match self {
// 			Self::Mode(mode) => mode.to_int(),
// 			Self::Number(num) => *num,
// 		}
// 	}
// }


/// Used for both difficulty and gamemode to have compatability with different versions
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum EnumOrNumber<T> {
	Enum(T),
	Num(i32),
}

impl <T: Display> Display for EnumOrNumber<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::Enum(e) => e.to_string(),
			Self::Num(num) => num.to_string(),
		})
	}
}

impl <T: ToInt> ToInt for EnumOrNumber<T> {
	fn to_int(&self) -> i32 {
		match self {
			Self::Enum(e) => e.to_int(),
			Self::Num(num) => *num,
		}
	}
}

/// Allow an enum or custom string
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum EnumOrString<T> {
	Enum(T),
	String(String),
}

impl <T: Display> Display for EnumOrString<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", match self {
			Self::Enum(e) => e.to_string(),
			Self::String(string) => string.clone(),
		})
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
