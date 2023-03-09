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

	pub fn to_bytes(&self) -> u32 {
		match self {
			Self::B(n) => *n,
			Self::Kb(n) => *n * 1024,
			Self::Mb(n) => *n * 1024 * 1024,
			Self::Gb(n) => *n * 1024 * 1024 * 1024
		}
	}

	pub fn avg(left: Self, right: Self) -> Self {
		Self::B((left.to_bytes() + right.to_bytes()) / 2)
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

/// Preset for generating game arguments (Usually for optimization)
#[derive(Debug)]
pub enum ArgsPreset {
	None,
	Aikars,
}

impl ArgsPreset {
	pub fn from_str(string: &str) -> Self {
		match string {
			"aikars" => Self::Aikars,
			"none" => Self::None,
			_ => Self::None,
		}
	}

	pub fn generate_args(&self, mem_avg: Option<MemoryNum>) -> Vec<String> {
		match self {
			Self::None => vec![],
			Self::Aikars => {
				let (
					new_size_percent,
					max_new_size_percent,
					heap_region_size,
					reserve_percent,
					ihop
				) = if let Some(avg) = mem_avg {
					if avg.to_bytes() > MemoryNum::Gb(12).to_bytes() {
						( "40", "50", "16M", "15", "20" )
					} else {
						( "30", "40", "8M", "20", "15" )
					}
				} else {
					( "30", "40", "8M", "20", "15" )
				};
				vec![
					String::from("-XX:+UseG1GC"),
					String::from("-XX:+ParallelRefProcEnabled"),
					String::from("-XX:MaxGCPauseMillis=200"),
					String::from("-XX:+UnlockExperimentalVMOptions"),
					String::from("-XX:+DisableExplicitGC"),
					String::from("-XX:+AlwaysPreTouch"),
					format!("-XX:G1NewSizePercent={new_size_percent}"),
					format!("-XX:G1MaxNewSizePercent={max_new_size_percent}"),
					format!("-XX:G1HeapRegionSize={heap_region_size}"),
					format!("-XX:G1ReservePercent={reserve_percent}"),
					String::from("-XX:G1HeapWastePercent=5"),
					String::from("-XX:G1MixedGCCountTarget=4"),
					format!("-XX:InitiatingHeapOccupancyPercent={ihop}"),
					String::from("-XX:G1MixedGCLiveThresholdPercent=90"),
					String::from("-XX:G1RSetUpdatingPauseTimePercent=5"),
					String::from("-XX:SurvivorRatio=32"),
					String::from("-XX:+PerfDisableSharedMem"),
					String::from("-XX:MaxTenuringThreshold=1"),
					String::from("-Dusing.aikars.flags=https://mcflags.emc.gs"),
					String::from("-Daikars.new.flags=true")
				]
			}
		}
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
