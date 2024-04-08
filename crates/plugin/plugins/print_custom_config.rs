use mcvm_plugin::api::{CustomPlugin, MCVMOutput, MessageContents, MessageLevel};

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::new("print_custom_config")?;
	plugin.on_load(|ctx, _| {
		if let Some(custom_config) = ctx.get_custom_config().map(String::from) {
			ctx.get_output().display(
				MessageContents::Property(
					"Config".into(),
					Box::new(MessageContents::Simple(custom_config)),
				),
				MessageLevel::Important,
			);
		}
		Ok(())
	})?;

	Ok(())
}
