use clap::Parser;
use color_print::cprintln;
use mcvm_plugin::api::CustomPlugin;
use mcvm_shared::util::open_link;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("webtools", include_str!("plugin.json"))?;
	plugin.subcommand(|_, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "webtool" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		let cli = Cli::parse_from(it);
		match cli.subcommand {
			Subcommand::List => list(),
			Subcommand::Open { tool } => open(&tool),
		}

		Ok(())
	})?;

	Ok(())
}

#[derive(clap::Parser)]
struct Cli {
	#[command(subcommand)]
	subcommand: Subcommand,
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
	#[command(about = "List all available tools")]
	#[command(alias = "ls")]
	List,
	#[command(about = "Open a webtool")]
	Open {
		/// The tool to open
		tool: WebTool,
	},
}

fn list() {
	for tool in ALL_WEBTOOLS {
		cprintln!("<s> - {tool} ({})", tool.name());
	}
}

fn open(webtool: &WebTool) {
	cprintln!("<s>Opening <b>{webtool}</> at <m>{}</>...", webtool.url());
	let _ = open_link(webtool.url());
}

macro_rules! define_webtools {
	($($tool:ident, $display_name:literal, $name:literal, $url:literal);+$(;)?) => {
		static ALL_WEBTOOLS: &[WebTool] = &[
			$(WebTool::$tool,)+
		];

		#[derive(Debug, Clone, Copy, clap::ValueEnum)]
		enum WebTool {
			$(
				$tool,
			)+
		}

		impl WebTool {
			fn url(&self) -> &'static str {
				match &self {
					$(
						Self::$tool => $url,
					)+
				}
			}

			fn name(&self) -> &'static str {
				match &self {
					$(
						Self::$tool => $name,
					)+
				}
			}
		}

		impl std::fmt::Display for WebTool {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "{}", match &self {
					$(
						Self::$tool => $display_name,
					)+
				})
			}
		}

		impl std::str::FromStr for WebTool {
			type Err = ();

			fn from_str(string: &str) -> Result<Self, Self::Err> {
				match string {
					$(
						$name => Ok(Self::$tool),
					)+
					_ => Err(()),
				}
			}
		}
	};
}

define_webtools! {
	Chunkbase, "Chunkbase", "chunkbase", "https://chunkbase.com";
	Wiki, "Minecraft Wiki", "wiki", "https://minecraft.wiki";
}
