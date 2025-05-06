use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use anyhow::Context;
use mcvm_plugin::api::CustomPlugin;

fn main() -> anyhow::Result<()> {
	let mut plugin =
		CustomPlugin::from_manifest_file("packhost", include_str!("plugin.json"))?;
	plugin.while_instance_launch(|_, arg| {
		let path = arg.custom_config.get("hosted_resource_pack");
		let Some(path) = path else {
			return Ok(());
		};
		let _path: String = serde_json::from_value(path.clone())
			.context("Failed to read hosted resource pack path")?;

		let listener = TcpListener::bind("127.0.0.1:80").context("Failed to bind TCP listener")?;

		for stream in listener.incoming() {
			let stream = stream.unwrap();

			handle_connection(stream);
		}

		Ok(())
	})?;

	Ok(())
}

fn handle_connection(mut stream: TcpStream) {
	let buf_reader = BufReader::new(&stream);
	let _: Vec<_> = buf_reader
		.lines()
		.map(|result| result.unwrap())
		.take_while(|line| !line.is_empty())
		.collect();

	let response = "HTTP/1.1 200 OK\r\n\r\n";

	stream.write_all(response.as_bytes()).unwrap();
}
