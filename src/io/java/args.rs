#[derive(Debug, Clone, PartialEq)]
pub enum MemoryNum {
	B(u32),
	Kb(u32),
	Mb(u32),
	Gb(u32),
}

impl MemoryNum {
	pub fn from_str(string: &str) -> Option<Self> {
		Some(match string.chars().last()? {
			'k' | 'K' => Self::Kb(string[..string.len() - 1].parse().ok()?),
			'm' | 'M' => Self::Mb(string[..string.len() - 1].parse().ok()?),
			'g' | 'G' => Self::Gb(string[..string.len() - 1].parse().ok()?),
			_ => Self::B(string.parse().ok()?),
		})
	}

	pub fn to_string(&self) -> String {
		match self {
			Self::B(n) => n.to_string(),
			Self::Kb(n) => n.to_string() + "k",
			Self::Mb(n) => n.to_string() + "m",
			Self::Gb(n) => n.to_string() + "g",
		}
	}
}

pub enum MemoryArg {
	Init,
	Max,
}

impl MemoryArg {
	pub fn to_string(&self, n: MemoryNum) -> String {
		let arg = match self {
			Self::Init => String::from("-Xms"),
			Self::Max => String::from("-Xmx"),
		};

		arg + &n.to_string()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_mem_parse() {
		assert_eq!(MemoryNum::from_str("2358"), Some(MemoryNum::B(2358)));
		assert_eq!(MemoryNum::from_str("0798m"), Some(MemoryNum::Mb(798)));
		assert_eq!(MemoryNum::from_str("1G"), Some(MemoryNum::Gb(1)));
		assert_eq!(MemoryNum::from_str("5a"), None);
		assert_eq!(MemoryNum::from_str("fooG"), None);
		assert_eq!(MemoryNum::from_str(""), None);
	}
}
