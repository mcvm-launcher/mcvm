use std::str::FromStr;

use clap::Parser;
use color_print::cprintln;
use mcvm_plugin::{api::CustomPlugin, hooks::SidebarButton};
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

	plugin.get_page(|_, page| {
		if !page.contains("webtools") {
			return Ok(None);
		}

		// Extract a specific tool
		if let Some(pos) = page.chars().position(|x| x == '-') {
			if pos == page.len() - 1 {
				return Ok(None);
			}
			let tool = &page[pos + 1..];
			let Ok(tool) = <WebTool as FromStr>::from_str(tool) else {
				return Ok(None);
			};

			let page = include_str!("tool_page.html");
			let page = page.replace("{{url}}", tool.url());
			return Ok(Some(page));
		}

		let page = include_str!("all_tools.html");
		let mut tools_string = String::new();
		for tool in ALL_WEBTOOLS {
			let component = include_str!("tool_component.html");
			let component = component.replace("{{id}}", tool.name());
			let component = component.replace("{{name}}", tool.display_name());
			let component = component.replace("{{description}}", tool.description());
			let url = if tool.embed_allowed() { "" } else { tool.url() };
			let component = component.replace("{{url}}", url);
			let component = component.replace("{{icon}}", tool.icon());
			tools_string.push_str(&component);
		}
		let page = page.replace("{{tools}}", &tools_string);

		Ok(Some(page))
	})?;

	plugin.add_sidebar_buttons(|_, _| {
		let icon = include_str!("gear.svg");
		Ok(vec![SidebarButton {
			html: format!("<div style=\"margin-top:0.3rem;margin-right:-0.2rem\">{icon}</div><div>Webtools</div>"),
			href: "/custom/webtools".into(),
			selected_url_start: Some("/custom/webtools".into()),
			color: "#1b48c4".into(),
			..Default::default()
		}])
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
	($($tool:ident, $display_name:literal, $name:literal, $url:literal, $description:literal, $icon:literal, $embed_allowed:literal);+$(;)?) => {
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

			fn display_name(&self) -> &'static str {
				match &self {
					$(
						Self::$tool => $display_name,
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

			fn description(&self) -> &'static str {
				match &self {
					$(
						Self::$tool => $description,
					)+
				}
			}

			fn icon(&self) -> &'static str {
				match &self {
					$(
						Self::$tool => $icon,
					)+
				}
			}

			fn embed_allowed(&self) -> bool {
				match &self {
					$(
						Self::$tool => $embed_allowed,
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
	Chunkbase, "Chunkbase", "chunkbase", "https://chunkbase.com", "Tool for mapping out Minecraft worlds", "https://chunkbase.com/favicon.ico", false;
	Wiki, "Minecraft Wiki", "wiki", "https://minecraft.wiki", "The official source for Minecraft information", "https://minecraft.wiki/images/wiki.png", false;
	McStacker, "MCStacker", "mcstacker", "https://mcstacker.net/", "Minecraft command generator", "https://mcstacker.net/favicon.ico", true;
}
