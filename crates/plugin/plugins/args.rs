use std::str::FromStr;

use anyhow::bail;
use mcvm_plugin::api::{CustomPlugin, MCVMOutput, MessageContents, MessageLevel};
use mcvm_plugin::hooks::ModifyInstanceConfigResult;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("args")?;
	plugin.modify_instance_config(|ctx, config| {
		let args = if let Some(preset) = config.get("args_preset") {
			if let Some(preset) = preset.as_str() {
				if let Ok(preset) = ArgsPreset::from_str(preset) {
					preset.generate_args()
				} else {
					ctx.get_output().display(
						MessageContents::Error("Invalid args preset".into()),
						MessageLevel::Important,
					);
					Vec::new()
				}
			} else {
				ctx.get_output().display(
					MessageContents::Error("Args preset must be a string".into()),
					MessageLevel::Important,
				);
				Vec::new()
			}
		} else {
			Vec::new()
		};

		Ok(ModifyInstanceConfigResult {
			additional_jvm_args: args,
		})
	})?;

	Ok(())
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
	pub fn generate_args(&self) -> Vec<String> {
		match self {
			Self::None => vec![],
			Self::Aikars => {
				let (
					new_size_percent,
					max_new_size_percent,
					heap_region_size,
					reserve_percent,
					ihop,
				) = ("40", "50", "16M", "15", "20");

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
