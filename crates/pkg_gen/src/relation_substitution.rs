use std::collections::HashMap;

use anyhow::bail;

/// Method for relation substitution in generated packages
pub enum RelationSubMethod {
	/// Don't substitute
	None,
	/// Map inputs to outputs
	Map(HashMap<String, String>),
	/// Run a function
	Function(Box<dyn Fn(&str) -> anyhow::Result<String>>),
}

impl RelationSubMethod {
	/// Substitutes a dependency using the given method
	pub fn substitute(&self, relation: &str) -> anyhow::Result<String> {
		match self {
			Self::None => Ok(relation.to_string()),
			Self::Map(map) => {
				if let Some(dep_id) = map.get(relation) {
					Ok(dep_id.clone())
				} else {
					bail!("Dependency {relation} was not substituted");
				}
			}
			Self::Function(function) => function(relation),
		}
	}
}
