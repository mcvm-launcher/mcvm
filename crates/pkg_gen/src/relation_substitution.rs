use std::collections::HashMap;

use anyhow::Context;

/// Asynchronous function for substituting relations
pub trait RelationSubFunction: AsyncFn(&str) -> anyhow::Result<String> {}

impl<A: AsyncFn(&str) -> anyhow::Result<String>> RelationSubFunction for A {}

/// Creates a None RelationSubMethod
pub fn relation_sub_none() -> impl RelationSubFunction {
	async |x| Ok(x.to_string())
}

/// Creates a Map RelationSubMethod
pub fn relation_sub_map(map: HashMap<String, String>) -> impl RelationSubFunction {
	async move |x| {
		map.get(x)
			.cloned()
			.with_context(|| format!("Dependency {x} was not substituted"))
	}
}
