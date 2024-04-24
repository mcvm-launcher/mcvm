use std::fmt::Display;

/// An amount of memory, used for Java memory arguments
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryNum {
	/// Bytes
	B(u32),
	/// Kilobytes
	Kb(u32),
	/// Megabytes
	Mb(u32),
	/// Gigabytes
	Gb(u32),
}

impl MemoryNum {
	/// Parse a string into a MemoryNum
	pub fn parse(string: &str) -> Option<Self> {
		Some(match string.chars().last()? {
			'k' | 'K' => Self::Kb(string[..string.len() - 1].parse().ok()?),
			'm' | 'M' => Self::Mb(string[..string.len() - 1].parse().ok()?),
			'g' | 'G' => Self::Gb(string[..string.len() - 1].parse().ok()?),
			_ => Self::B(string.parse().ok()?),
		})
	}

	/// Converts into the equivalent amount in bytes
	pub fn to_bytes(&self) -> u32 {
		match self {
			Self::B(n) => *n,
			Self::Kb(n) => *n * 1024,
			Self::Mb(n) => *n * 1024 * 1024,
			Self::Gb(n) => *n * 1024 * 1024 * 1024,
		}
	}

	/// Averages two amounts of memory
	pub fn avg(left: Self, right: Self) -> Self {
		Self::B((left.to_bytes() + right.to_bytes()) / 2)
	}
}

impl Display for MemoryNum {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::B(n) => n.to_string(),
				Self::Kb(n) => n.to_string() + "k",
				Self::Mb(n) => n.to_string() + "m",
				Self::Gb(n) => n.to_string() + "g",
			}
		)
	}
}

/// Different types of Java memory arguments
pub enum MemoryArg {
	/// Minimum heap size
	Min,
	/// Maximum heap size
	Max,
}

impl MemoryArg {
	/// Convert this memory arg to an argument string with a memory num
	pub fn to_string(&self, n: MemoryNum) -> String {
		let arg = match self {
			Self::Min => "-Xms".to_string(),
			Self::Max => "-Xmx".to_string(),
		};

		arg + &n.to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_mem_parse() {
		assert_eq!(MemoryNum::parse("2358"), Some(MemoryNum::B(2358)));
		assert_eq!(MemoryNum::parse("0798m"), Some(MemoryNum::Mb(798)));
		assert_eq!(MemoryNum::parse("1G"), Some(MemoryNum::Gb(1)));
		assert_eq!(MemoryNum::parse("5a"), None);
		assert_eq!(MemoryNum::parse("fooG"), None);
		assert_eq!(MemoryNum::parse(""), None);
	}

	#[test]
	fn test_mem_arg_output() {
		assert_eq!(
			MemoryArg::Max.to_string(MemoryNum::Gb(4)),
			"-Xmx4g".to_string()
		);
		assert_eq!(
			MemoryArg::Min.to_string(MemoryNum::B(128)),
			"-Xms128".to_string()
		);
	}
}
