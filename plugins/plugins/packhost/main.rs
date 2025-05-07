use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;

use anyhow::Context;
use mcvm_plugin::api::CustomPlugin;
use serde::Deserialize;

fn main() -> anyhow::Result<()> {
	let mut plugin = CustomPlugin::from_manifest_file("packhost", include_str!("plugin.json"))?;
	plugin.while_instance_launch(|_, arg| {
		let config = arg.config.common.plugin_config.get("packhost");
		let Some(config) = config else {
			return Ok(());
		};
		let config: Config = serde_json::from_value(config.clone())
			.context("Failed to read hosted resource pack config")?;

		let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port.unwrap_or(8080)))
			.context("Failed to bind TCP listener")?;

		// Figure out the hash of the file
		let path = PathBuf::from(config.path);
		let mut file =
			BufReader::new(File::open(path).context("Failed to open resource pack file")?);

		for stream in listener.incoming() {
			let stream = stream.unwrap();

			let _ = handle_connection(stream, &mut file);
		}

		Ok(())
	})?;

	Ok(())
}

fn handle_connection(mut stream: TcpStream, _file: &mut BufReader<File>) -> anyhow::Result<()> {
	let buf_reader = BufReader::new(&stream);
	let _: Vec<_> = buf_reader
		.lines()
		.map(|result| result.unwrap())
		.take_while(|line| !line.is_empty())
		.collect();

	let status_line = "HTTP/1.1 200 OK\r\n";
	let headers = "Content-Type:application/zip\r\n";

	stream
		.write_all(status_line.as_bytes())
		.context("Failed to write status line")?;
	stream
		.write_all(headers.as_bytes())
		.context("Failed to write headers")?;

	Ok(())
}

#[derive(Deserialize)]
struct Config {
	pub path: String,
	pub port: Option<u16>,
}
