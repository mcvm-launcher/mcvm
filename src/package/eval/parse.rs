use mcvm_parse::parse::lex_and_parse;

use super::super::Package;
use crate::io::files::paths::Paths;

impl Package {
	/// Parse the contents of the package
	pub async fn parse(&mut self, paths: &Paths) -> anyhow::Result<()> {
		self.ensure_loaded(paths, false).await?;
		let data = self.data.get_mut();
		if !data.parsed.is_empty() {
			return Ok(());
		}

		let parsed = lex_and_parse(&data.contents)?;

		data.parsed.fill(parsed);

		Ok(())
	}
}
