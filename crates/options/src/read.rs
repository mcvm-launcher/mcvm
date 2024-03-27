use std::{collections::HashMap, fmt::Display, io::Read};

use anyhow::{anyhow, Context};
#[cfg(feature = "schema")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use mcvm_shared::util::ToInt;

use super::Options;

pub fn parse_options<R: Read>(reader: &mut R) -> anyhow::Result<Options> {
	serde_json::from_reader(reader).context("Failed to parse options")
}

#[cfg(test)]
pub fn parse_options_str(string: &str) -> anyhow::Result<Options> {
	serde_json::from_str(string).context("Failed to parse options")
}

/// Collect a hashmap from an existing options file so we can compare with it
pub fn read_options_file(
	contents: &str,
	separator: char,
) -> anyhow::Result<HashMap<String, String>> {
	// TODO: Make this more robust to formatting differences and whitespace
	let mut out = HashMap::new();
	for (i, line) in contents.lines().enumerate() {
		if !line.contains(separator) {
			continue;
		}
		let index = line
			.find(separator)
			.ok_or(anyhow!("Options line {i} does not have a colon separator!"))?;
		let (key, value) = line.split_at(index);
		out.insert(key.to_string(), String::from(&value[1..]));
	}

	Ok(out)
}

/// Used for both difficulty and gamemode to have compatability with different versions
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum EnumOrNumber<T> {
	Enum(T),
	Num(i32),
}

impl<T: Display> Display for EnumOrNumber<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Enum(e) => e.to_string(),
				Self::Num(num) => num.to_string(),
			}
		)
	}
}

impl<T: ToInt> ToInt for EnumOrNumber<T> {
	fn to_int(&self) -> i32 {
		match self {
			Self::Enum(e) => e.to_int(),
			Self::Num(num) => *num,
		}
	}
}

/// Allow an enum or custom string
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[serde(untagged)]
pub enum EnumOrString<T> {
	Enum(T),
	String(String),
}

impl<T: Display> Display for EnumOrString<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Enum(e) => e.to_string(),
				Self::String(string) => string.clone(),
			}
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_default() {
		parse_options_str("{}").unwrap();
	}

	#[test]
	fn test_enums() {
		#[derive(Clone)]
		enum TestEnum {
			Foo,
			Bar,
		}

		impl Display for TestEnum {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(
					f,
					"{}",
					match self {
						Self::Foo => "foo",
						Self::Bar => "bar",
					}
				)
			}
		}

		impl ToInt for TestEnum {
			fn to_int(&self) -> i32 {
				self.clone() as i32
			}
		}

		assert_eq!(EnumOrNumber::Enum(TestEnum::Foo).to_int(), 0);
		assert_eq!(EnumOrNumber::Enum(TestEnum::Bar).to_string(), "bar");
		assert_eq!(EnumOrNumber::Enum(TestEnum::Foo).to_int().to_string(), "0");
		assert_eq!(EnumOrString::Enum(TestEnum::Bar).to_string(), "bar");
	}

	#[test]
	fn test_read_options_file() -> anyhow::Result<()> {
		let text = r#"
fov=12
hello=world

yes=false
		"#;
		let options = read_options_file(text, '=')?;

		assert_eq!(options.get("fov").unwrap(), "12");
		assert_eq!(options.get("hello").unwrap(), "world");
		assert_eq!(options.get("yes").unwrap(), "false");

		Ok(())
	}
}
