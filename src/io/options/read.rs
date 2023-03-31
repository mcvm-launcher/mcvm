use std::{io::Read, fmt::Display};

use anyhow::Context;
use serde::Deserialize;

use crate::util::ToInt;

use super::Options;

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

	#[derive(Clone)]
	enum TestEnum {
		Foo,
		Bar,
	}

	impl Display for TestEnum {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			write!(f, "{}", match self {
				Self::Foo => "foo",
				Self::Bar => "bar",
			})
		}
	}

	impl ToInt for TestEnum {
		fn to_int(&self) -> i32 {
			self.clone() as i32
		}
	}

	#[test]
	fn test_enums() {
		assert_eq!(EnumOrNumber::Enum(TestEnum::Foo).to_int(), 0);
		assert_eq!(EnumOrNumber::Enum(TestEnum::Bar).to_string(), "bar");
		assert_eq!(EnumOrNumber::Enum(TestEnum::Foo).to_int().to_string(), "0");
		assert_eq!(EnumOrString::Enum(TestEnum::Bar).to_string(), "bar");
	}
}
