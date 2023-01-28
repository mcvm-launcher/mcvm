use serde_json::Value;

pub type JsonObject = serde_json::Map<String, Value>;

#[derive(Debug)]
pub enum JsonType {
	Int,
	Float,
	Bool,
	Str,
	Array,
	Object
}

impl std::fmt::Display for JsonType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			JsonType::Int => write!(f, "Integer"),
			JsonType::Float => write!(f, "Float"),
			JsonType::Bool => write!(f, "Bool"),
			JsonType::Str => write!(f, "String"),
			JsonType::Array => write!(f, "Array"),
			JsonType::Object => write!(f, "Object")
		}
	}
}

#[derive(Debug, thiserror::Error)]
pub enum JsonError {
	#[error("{}", .0)]
	Parse(#[from] serde_json::Error),
	#[error("Key [{}] was not found in object", .0)]
	Key(String),
	#[error("Key [{}] was expected to be of type {}", .0, .1)]
	KeyType(String, JsonType),
	#[error("Value was expected to be of type {}", .0)]
	Type(JsonType),
	#[error("Array index [{}] out of range [{}]", .0, .1)]
	Index(usize, usize)
}

pub fn parse_json(contents: &str) -> Result<Box<Value>, JsonError> {
	let doc: Value = serde_json::from_str(contents)?;
	Ok(Box::new(doc))
}

pub fn parse_object<'a>(contents: &str) -> Result<Box<JsonObject>, JsonError> {
	let doc: Value = serde_json::from_str(contents)?;
	let obj = ensure_type(doc.as_object(), JsonType::Object)?;
	Ok(Box::new(obj.clone()))
}

pub fn access_i64(obj: &JsonObject, key: &str) -> Result<i64, JsonError> {
	match obj.get(key) {
		Some(val) => match val.as_i64() {
			Some(val) => Ok(val),
			None => Err(JsonError::KeyType(key.to_string(), JsonType::Int))
		},
		None => Err(JsonError::Key(key.to_string()))
	}
}

pub fn access_f64(obj: &JsonObject, key: &str) -> Result<f64, JsonError> {
	match obj.get(key) {
		Some(val) => match val.as_f64() {
			Some(val) => Ok(val),
			None => Err(JsonError::KeyType(key.to_string(), JsonType::Float))
		},
		None => Err(JsonError::Key(key.to_string()))
	}
}

pub fn access_bool(obj: &JsonObject, key: &str) -> Result<bool, JsonError> {
	match obj.get(key) {
		Some(val) => match val.as_bool() {
			Some(val) => Ok(val),
			None => Err(JsonError::KeyType(key.to_string(), JsonType::Bool))
		},
		None => Err(JsonError::Key(key.to_string()))
	}
}

pub fn access_str<'a>(obj: &'a JsonObject, key: &str) -> Result<&'a str, JsonError> {
	match obj.get(key) {
		Some(val) => match val.as_str() {
			Some(val) => Ok(val),
			None => Err(JsonError::KeyType(key.to_string(), JsonType::Str))
		},
		None => Err(JsonError::Key(key.to_string()))
	}
}

pub fn access_array<'a>(obj: &'a JsonObject, key: &str) -> Result<&'a Vec<Value>, JsonError> {
	match obj.get(key) {
		Some(val) => match val.as_array() {
			Some(val) => Ok(val),
			None => Err(JsonError::KeyType(key.to_string(), JsonType::Array))
		},
		None => Err(JsonError::Key(key.to_string()))
	}
}

pub fn access_object<'a>(obj: &'a JsonObject, key: &str) -> Result<&'a JsonObject, JsonError> {
	match obj.get(key) {
		Some(val) => match val.as_object() {
			Some(val) => Ok(val),
			None => Err(JsonError::KeyType(key.to_string(), JsonType::Object))
		},
		None => Err(JsonError::Key(key.to_string()))
	}
}

// Used after getting a type to create an error if the type conversion failed
pub fn ensure_type<T>(value: Option<T>, typ: JsonType) -> Result<T, JsonError> {
	match value {
		Some(val) => Ok(val),
		None => Err(JsonError::Type(typ))
	}
}

// Json access with an assertion
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! access {
	($obj:expr, $key:expr, $typ:ident) => {
		concat_idents!(access_, $typ)($obj, $key)?
	};
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! access {
	($obj:expr, $key:expr, $typ:ident) => {
		unsafe {
			concat_idents!(access_, $typ)($obj, $key).unwrap_unchecked()
		}
	};
}
