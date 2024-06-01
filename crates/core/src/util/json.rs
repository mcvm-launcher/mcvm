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

/// Utility function to merge serde_json values
pub fn merge(a: &mut Value, b: Value) {
	if let Value::Object(a) = a {
		if let Value::Object(b) = b {
			merge_objects(a, b);
			return;
		}
	}

	*a = b;
}

/// Utility function to merge serde_json objects
pub fn merge_objects(a: &mut serde_json::Map<String, Value>, b: serde_json::Map<String, Value>) {
	for (k, v) in b {
		if v.is_null() {
			a.remove(&k);
		} else {
			merge(a.entry(k).or_insert(Value::Null), v);
		}
	}
}
