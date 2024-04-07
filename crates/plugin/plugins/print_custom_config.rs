use mcvm_plugin::api::CustomPlugin;

fn main() -> anyhow::Result<()> {
	let plugin = CustomPlugin::new("print_custom_config")?;
	plugin.on_load(|ctx, _| {
		eprintln!("Config: {}", ctx.get_custom_config().unwrap_or_default());
		Ok(())
	})?;

	Ok(())
}
