use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ServerOptions {

}

impl Default for ServerOptions {
	fn default() -> Self {
		Self {}
	}
}
