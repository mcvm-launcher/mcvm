use std::collections::HashMap;

use anyhow::bail;

/// Asynchronous function for substituting relations
pub trait RelationSubFunction: AsyncFn(&str) -> anyhow::Result<String> {}

impl<A: AsyncFn(&str) -> anyhow::Result<String>> RelationSubFunction for A {}

/// Method for relation substitution in generated packages
pub enum RelationSubMethod<A: RelationSubFunction> {
	/// Don't substitute
	None,
	/// Map inputs to outputs
	Map(HashMap<String, String>),
	/// Run a function
	Function(A),
}

// Some trickery since we cant implement the AsyncFn trait in stable rust

/// Creates a None RelationSubMethod
pub fn relation_sub_none() -> RelationSubMethod<impl RelationSubFunction> {
	#[allow(unused_assignments)]
	let mut x = RelationSubMethod::Function(async |x| Ok(x.to_string()));
	x = RelationSubMethod::None;
	x
}

/// Creates a Map RelationSubMethod
pub fn relation_sub_map(map: HashMap<String, String>) -> RelationSubMethod<impl RelationSubFunction> {
	#[allow(unused_assignments)]
	let mut x = RelationSubMethod::Function(async |x| Ok(x.to_string()));
	x = RelationSubMethod::Map(map);
	x
}

impl<A: RelationSubFunction> RelationSubMethod<A> {
	/// Substitutes a dependency using the given method
	pub async fn substitute(&self, relation: &str) -> anyhow::Result<String> {
		match self {
			Self::None => Ok(relation.to_string()),
			Self::Map(map) => {
				if let Some(dep_id) = map.get(relation) {
					Ok(dep_id.clone())
				} else {
					bail!("Dependency {relation} was not substituted");
				}
			}
			Self::Function(function) => function(relation).await,
		}
	}
}
