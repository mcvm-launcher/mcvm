use std::{fmt::Display, str::FromStr};

use anyhow::bail;

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
			Self::Min => String::from("-Xms"),
			Self::Max => String::from("-Xmx"),
		};

		arg + &n.to_string()
	}
}

/// Preset for generating game arguments (Usually for optimization)
#[derive(Debug)]
pub enum ArgsPreset {
	/// No preset
	None,
	/// Aikar's args
	Aikars,
	/// Krusic's args
	Krusic,
	/// Obydux's args
	Obydux,
}

impl FromStr for ArgsPreset {
	type Err = anyhow::Error;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"aikars" => Ok(Self::Aikars),
			"krusic" => Ok(Self::Krusic),
			"obydux" => Ok(Self::Obydux),
			"none" => Ok(Self::None),
			_ => bail!("Unknown argument preset '{s}'"),
		}
	}
}

impl ArgsPreset {
	/// Generate the JVM arguments for this arguments preset
	pub fn generate_args(&self, mem_avg: Option<MemoryNum>) -> Vec<String> {
		match self {
			Self::None => vec![],
			Self::Aikars => {
				let (
					new_size_percent,
					max_new_size_percent,
					heap_region_size,
					reserve_percent,
					ihop,
				) = if let Some(avg) = mem_avg {
					if avg.to_bytes() > MemoryNum::Gb(12).to_bytes() {
						("40", "50", "16M", "15", "20")
					} else {
						("30", "40", "8M", "20", "15")
					}
				} else {
					("30", "40", "8M", "20", "15")
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
					String::from("-Daikars.new.flags=true"),
				]
			}
			Self::Krusic => vec![
				String::from("-XX:+UnlockExperimentalVMOptions"),
				String::from("-XX:+DisableExplicitGC"),
				String::from("-XX:-UseParallelGC"),
				String::from("-XX:-UseG1GC"),
				String::from("-XX:+UseZGC"),
			],
			Self::Obydux => vec![
				String::from("-XX:+UnlockExperimentalVMOptions"),
				String::from("-XX:+UnlockDiagnosticVMOptions"),
				String::from("-Dterminal.jline=false"),
				String::from("-Dterminal.ansi=true"),
				String::from("-Djline.terminal=jline.UnsupportedTerminal"),
				String::from("-Dlog4j2.formatMsgNoLookups=true"),
				String::from("-XX:+AlwaysActAsServerClassMachine"),
				String::from("-XX:+AlwaysPreTouch"),
				String::from("-XX:+DisableExplicitGC"),
				String::from("-XX:+UseNUMA"),
				String::from("-XX:AllocatePrefetchStyle=3"),
				String::from("-XX:NmethodSweepActivity=1"),
				String::from("-XX:ReservedCodeCacheSize=400M"),
				String::from("-XX:NonNMethodCodeHeapSize=12M"),
				String::from("-XX:ProfiledCodeHeapSize=194M"),
				String::from("-XX:NonProfiledCodeHeapSize=194M"),
				String::from("-XX:+PerfDisableSharedMem"),
				String::from("-XX:+UseFastUnorderedTimeStamps"),
				String::from("-XX:+UseCriticalJavaThreadPriority"),
				String::from("-XX:+EagerJVMCI"),
				String::from("-Dgraal.TuneInlinerExploration=1"),
				String::from("-Dgraal.CompilerConfiguration=enterprise"),
				String::from("-XX:+UseG1GC"),
				String::from("-XX:+ParallelRefProcEnabled"),
				String::from("-XX:MaxGCPauseMillis=200"),
				String::from("-XX:+UnlockExperimentalVMOptions"),
				String::from("-XX:+UnlockDiagnosticVMOptions"),
				String::from("-XX:+DisableExplicitGC"),
				String::from("-XX:+AlwaysPreTouch"),
				String::from("-XX:G1NewSizePercent=30"),
				String::from("-XX:G1MaxNewSizePercent=40"),
				String::from("-XX:G1HeapRegionSize=8M"),
				String::from("-XX:G1ReservePercent=20"),
				String::from("-XX:G1HeapWastePercent=5"),
				String::from("-XX:G1MixedGCCountTarget=4"),
				String::from("-XX:InitiatingHeapOccupancyPercent=15"),
				String::from("-XX:G1MixedGCLiveThresholdPercent=90"),
				String::from("-XX:G1RSetUpdatingPauseTimePercent=5"),
				String::from("-XX:SurvivorRatio=32"),
				String::from("-XX:+PerfDisableSharedMem"),
				String::from("-XX:MaxTenuringThreshold=1"),
				String::from("-XX:-UseBiasedLocking"),
				String::from("-XX:+UseStringDeduplication"),
				String::from("-XX:+UseFastUnorderedTimeStamps"),
				String::from("-XX:+UseAES"),
				String::from("-XX:+UseAESIntrinsics"),
				String::from("-XX:+UseFMA"),
				String::from("-XX:+UseLoopPredicate"),
				String::from("-XX:+RangeCheckElimination"),
				String::from("-XX:+EliminateLocks"),
				String::from("-XX:+DoEscapeAnalysis"),
				String::from("-XX:+UseCodeCacheFlushing"),
				String::from("-XX:+SegmentedCodeCache"),
				String::from("-XX:+UseFastJNIAccessors"),
				String::from("-XX:+OptimizeStringConcat"),
				String::from("-XX:+UseCompressedOops"),
				String::from("-XX:+UseThreadPriorities"),
				String::from("-XX:+OmitStackTraceInFastThrow"),
				String::from("-XX:+TrustFinalNonStaticFields"),
				String::from("-XX:ThreadPriorityPolicy=1"),
				String::from("-XX:+UseInlineCaches"),
				String::from("-XX:+RewriteBytecodes"),
				String::from("-XX:+RewriteFrequentPairs"),
				String::from("-XX:+UseNUMA"),
				String::from("-XX:-DontCompileHugeMethods"),
				String::from("-XX:+UseFPUForSpilling"),
				String::from("-XX:+UseVectorCmov"),
				String::from("-XX:+UseXMMForArrayCopy"),
				String::from("-XX:+UseTransparentHugePages"),
				String::from("-XX:+UseLargePages"),
				String::from("-Dfile.encoding=UTF-8"),
				String::from("-Xlog:async"),
				String::from("--add-modules"),
				String::from("jdk.incubator.vector"),
			],
		}
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
}
