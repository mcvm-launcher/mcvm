#![allow(dead_code)]
use serde_json::{json, Value};

/// A JSON map of strings to values
pub type JsonObject = serde_json::Map<String, Value>;

/// A type available in JSON
#[derive(Debug)]
pub enum JsonType {
	/// A JSON integer
	Int,
	/// A JSON float
	Float,
	/// A JSON boolean
	Bool,
	/// A JSON string
	Str,
	/// A JSON array
	Arr,
	/// A JSON object
	Obj,
}

impl std::fmt::Display for JsonType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			JsonType::Int => write!(f, "Integer"),
			JsonType::Float => write!(f, "Float"),
			JsonType::Bool => write!(f, "Bool"),
			JsonType::Str => write!(f, "String"),
			JsonType::Arr => write!(f, "Array"),
			JsonType::Obj => write!(f, "Object"),
		}
	}
}

/// Error returned from JSON manipulation functions
#[derive(Debug, thiserror::Error)]
pub enum JsonError {
	/// Error when parsing JSON fails
	#[error("{}", .0)]
	Parse(#[from] serde_json::Error),
	/// Error when a key is not found in an object
	#[error("Key [{}] was not found in object", .0)]
	Key(String),
	/// Error when the value of a key is not of the right type
	#[error("Key [{}] was expected to be of type {}", .0, .1)]
	KeyType(String, JsonType),
	/// Error when a value is not of the right type
	#[error("Value was expected to be of type {:?}", .0)]
	Type(Vec<JsonType>),
}

/// Parse a string into a JSON object
pub fn parse_object(contents: &str) -> Result<Box<JsonObject>, JsonError> {
	let doc: Value = serde_json::from_str(contents)?;
	let obj = ensure_type(doc.as_object(), JsonType::Obj)?;
	Ok(Box::new(obj.clone()))
}

/// Access an i32 from a serde_json object
pub fn access_i64(obj: &JsonObject, key: &str) -> Result<i64, JsonError> {
	obj.get(key)
		.ok_or(JsonError::Key(key.to_string()))?
		.as_i64()
		.ok_or(JsonError::KeyType(key.to_string(), JsonType::Int))
}

/// Access an f64 from a serde_json object
pub fn access_f64(obj: &JsonObject, key: &str) -> Result<f64, JsonError> {
	obj.get(key)
		.ok_or(JsonError::Key(key.to_string()))?
		.as_f64()
		.ok_or(JsonError::KeyType(key.to_string(), JsonType::Float))
}

/// Access a bool from a serde_json object
pub fn access_bool(obj: &JsonObject, key: &str) -> Result<bool, JsonError> {
	obj.get(key)
		.ok_or(JsonError::Key(key.to_string()))?
		.as_bool()
		.ok_or(JsonError::KeyType(key.to_string(), JsonType::Bool))
}

/// Access a str from a serde_json object
pub fn access_str<'a>(obj: &'a JsonObject, key: &str) -> Result<&'a str, JsonError> {
	obj.get(key)
		.ok_or(JsonError::Key(key.to_string()))?
		.as_str()
		.ok_or(JsonError::KeyType(key.to_string(), JsonType::Str))
}

/// Access an array from a serde_json object
pub fn access_array<'a>(obj: &'a JsonObject, key: &str) -> Result<&'a Vec<Value>, JsonError> {
	obj.get(key)
		.ok_or(JsonError::Key(key.to_string()))?
		.as_array()
		.ok_or(JsonError::KeyType(key.to_string(), JsonType::Arr))
}

/// Access an object from a serde_json object
pub fn access_object<'a>(obj: &'a JsonObject, key: &str) -> Result<&'a JsonObject, JsonError> {
	obj.get(key)
		.ok_or(JsonError::Key(key.to_string()))?
		.as_object()
		.ok_or(JsonError::KeyType(key.to_string(), JsonType::Obj))
}

/// Used after getting a type to create an error if the type conversion failed
pub fn ensure_type<T>(value: Option<T>, typ: JsonType) -> Result<T, JsonError> {
	value.ok_or(JsonError::Type(vec![typ]))
}

/// Returns an empty json object
pub fn empty_object() -> JsonObject {
	json!({})
		.as_object()
		.expect("Should be an empty object")
		.clone()
}
