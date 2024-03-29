use serde_json::{json, Value};

/// A JSON map of strings to values
pub type JsonObject = serde_json::Map<String, Value>;

/// Returns an empty json object
pub fn empty_object() -> JsonObject {
	json!({})
		.as_object()
		.expect("Should be an empty object")
		.clone()
}

/// Formats a JSON string to a pretty JSON string
pub fn format_json(text: &str) -> Result<String, serde_json::Error> {
	let into: Value = serde_json::from_str(text)?;
	let out = serde_json::to_string_pretty(&into)?;
	Ok(out)
}
