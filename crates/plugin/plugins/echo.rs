use mcvm_plugin::api::CustomPlugin;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("echo")?;
	plugin.subcommand(|ctx, args| {
		let _ = ctx;
		if let Some(sub) = args.first() {
			if sub == "echo" {
				if let Some(msg) = args.get(1) {
					println!("{msg}");
				}
			}
		}
		Ok(())
	})?;

	Ok(())
}
