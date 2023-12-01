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
			Self::Min => "-Xms".to_string(),
			Self::Max => "-Xmx".to_string(),
		};

		arg + &n.to_string()
	}
}

/// Preset for generating game arguments (Usually for optimization)
#[derive(Debug, Clone, Copy)]
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
					"-XX:+UseG1GC".to_string(),
					"-XX:+ParallelRefProcEnabled".to_string(),
					"-XX:MaxGCPauseMillis=200".to_string(),
					"-XX:+UnlockExperimentalVMOptions".to_string(),
					"-XX:+DisableExplicitGC".to_string(),
					"-XX:+AlwaysPreTouch".to_string(),
					format!("-XX:G1NewSizePercent={new_size_percent}"),
					format!("-XX:G1MaxNewSizePercent={max_new_size_percent}"),
					format!("-XX:G1HeapRegionSize={heap_region_size}"),
					format!("-XX:G1ReservePercent={reserve_percent}"),
					"-XX:G1HeapWastePercent=5".to_string(),
					"-XX:G1MixedGCCountTarget=4".to_string(),
					format!("-XX:InitiatingHeapOccupancyPercent={ihop}"),
					"-XX:G1MixedGCLiveThresholdPercent=90".to_string(),
					"-XX:G1RSetUpdatingPauseTimePercent=5".to_string(),
					"-XX:SurvivorRatio=32".to_string(),
					"-XX:+PerfDisableSharedMem".to_string(),
					"-XX:MaxTenuringThreshold=1".to_string(),
					"-Dusing.aikars.flags=https://mcflags.emc.gs".to_string(),
					"-Daikars.new.flags=true".to_string(),
				]
			}
			Self::Krusic => vec![
				"-XX:+UnlockExperimentalVMOptions".to_string(),
				"-XX:+DisableExplicitGC".to_string(),
				"-XX:-UseParallelGC".to_string(),
				"-XX:-UseG1GC".to_string(),
				"-XX:+UseZGC".to_string(),
			],
			Self::Obydux => vec![
				"-XX:+UnlockExperimentalVMOptions".to_string(),
				"-XX:+UnlockDiagnosticVMOptions".to_string(),
				"-Dterminal.jline=false".to_string(),
				"-Dterminal.ansi=true".to_string(),
				"-Djline.terminal=jline.UnsupportedTerminal".to_string(),
				"-Dlog4j2.formatMsgNoLookups=true".to_string(),
				"-XX:+AlwaysActAsServerClassMachine".to_string(),
				"-XX:+AlwaysPreTouch".to_string(),
				"-XX:+DisableExplicitGC".to_string(),
				"-XX:+UseNUMA".to_string(),
				"-XX:AllocatePrefetchStyle=3".to_string(),
				"-XX:NmethodSweepActivity=1".to_string(),
				"-XX:ReservedCodeCacheSize=400M".to_string(),
				"-XX:NonNMethodCodeHeapSize=12M".to_string(),
				"-XX:ProfiledCodeHeapSize=194M".to_string(),
				"-XX:NonProfiledCodeHeapSize=194M".to_string(),
				"-XX:+PerfDisableSharedMem".to_string(),
				"-XX:+UseFastUnorderedTimeStamps".to_string(),
				"-XX:+UseCriticalJavaThreadPriority".to_string(),
				"-XX:+EagerJVMCI".to_string(),
				"-Dgraal.TuneInlinerExploration=1".to_string(),
				"-Dgraal.CompilerConfiguration=enterprise".to_string(),
				"-XX:+UseG1GC".to_string(),
				"-XX:+ParallelRefProcEnabled".to_string(),
				"-XX:MaxGCPauseMillis=200".to_string(),
				"-XX:+UnlockExperimentalVMOptions".to_string(),
				"-XX:+UnlockDiagnosticVMOptions".to_string(),
				"-XX:+DisableExplicitGC".to_string(),
				"-XX:+AlwaysPreTouch".to_string(),
				"-XX:G1NewSizePercent=30".to_string(),
				"-XX:G1MaxNewSizePercent=40".to_string(),
				"-XX:G1HeapRegionSize=8M".to_string(),
				"-XX:G1ReservePercent=20".to_string(),
				"-XX:G1HeapWastePercent=5".to_string(),
				"-XX:G1MixedGCCountTarget=4".to_string(),
				"-XX:InitiatingHeapOccupancyPercent=15".to_string(),
				"-XX:G1MixedGCLiveThresholdPercent=90".to_string(),
				"-XX:G1RSetUpdatingPauseTimePercent=5".to_string(),
				"-XX:SurvivorRatio=32".to_string(),
				"-XX:+PerfDisableSharedMem".to_string(),
				"-XX:MaxTenuringThreshold=1".to_string(),
				"-XX:-UseBiasedLocking".to_string(),
				"-XX:+UseStringDeduplication".to_string(),
				"-XX:+UseFastUnorderedTimeStamps".to_string(),
				"-XX:+UseAES".to_string(),
				"-XX:+UseAESIntrinsics".to_string(),
				"-XX:+UseFMA".to_string(),
				"-XX:+UseLoopPredicate".to_string(),
				"-XX:+RangeCheckElimination".to_string(),
				"-XX:+EliminateLocks".to_string(),
				"-XX:+DoEscapeAnalysis".to_string(),
				"-XX:+UseCodeCacheFlushing".to_string(),
				"-XX:+SegmentedCodeCache".to_string(),
				"-XX:+UseFastJNIAccessors".to_string(),
				"-XX:+OptimizeStringConcat".to_string(),
				"-XX:+UseCompressedOops".to_string(),
				"-XX:+UseThreadPriorities".to_string(),
				"-XX:+OmitStackTraceInFastThrow".to_string(),
				"-XX:+TrustFinalNonStaticFields".to_string(),
				"-XX:ThreadPriorityPolicy=1".to_string(),
				"-XX:+UseInlineCaches".to_string(),
				"-XX:+RewriteBytecodes".to_string(),
				"-XX:+RewriteFrequentPairs".to_string(),
				"-XX:+UseNUMA".to_string(),
				"-XX:-DontCompileHugeMethods".to_string(),
				"-XX:+UseFPUForSpilling".to_string(),
				"-XX:+UseVectorCmov".to_string(),
				"-XX:+UseXMMForArrayCopy".to_string(),
				"-XX:+UseTransparentHugePages".to_string(),
				"-XX:+UseLargePages".to_string(),
				"-Dfile.encoding=UTF-8".to_string(),
				"-Xlog:async".to_string(),
				"--add-modules".to_string(),
				"jdk.incubator.vector".to_string(),
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
