mod docs;

use anyhow::{bail, Context};
use clap::Parser;
use color_print::{cprint, cprintln};
use docs::Docs;
use mcvm_plugin::api::CustomPlugin;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("docs")?;
	plugin.subcommand(|_, args| {
		let Some(subcommand) = args.first() else {
			return Ok(());
		};
		if subcommand != "docs" {
			return Ok(());
		}
		// Trick the parser to give it the right bin name
		let it = std::iter::once(format!("mcvm {subcommand}")).chain(args.into_iter().skip(1));
		let cli = Cli::parse_from(it);
		display_docs(cli.page)?;

		Ok(())
	})?;

	Ok(())
}

#[derive(clap::Parser)]
struct Cli {
	/// The documentation page to view. If omitted, lists the available pages
	page: Option<String>,
}

/// Display docs
fn display_docs(page: Option<String>) -> anyhow::Result<()> {
	let docs = Docs::load().context("Failed to load documentation")?;
	if let Some(page) = page {
		if let Some(page) = docs.get_page(&page) {
			termimad::print_text(page);
		} else {
			bail!("Documentation page '{page}' does not exist");
		}
	} else {
		let pages = docs.get_pages();
		cprintln!("<s>Available documentation pages:");
		for page in pages {
			cprint!("<k!> - </>");
			cprintln!("<b>{page}");
		}
	}
	Ok(())
}
