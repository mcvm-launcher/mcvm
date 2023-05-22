use anyhow::{bail, Context};

use super::conditions::Condition;
use super::conditions::ConditionKind;
use super::instruction::{parse_arg, InstrKind, Instruction};
use super::lex::{lex, reduce_tokens, Side, Token, TokenAndPos};
use super::Value;
use mcvm_shared::addon::AddonKind;

use std::collections::HashMap;

static DEFAULT_ROUTINE: &str = "__default__";

/// The type we use to index blocks in the hashmap
pub type BlockId = u16;

/// A list of instructions inside a routine or nested block (such as an if block)
#[derive(Debug, Clone)]
pub struct Block {
	pub contents: Vec<Instruction>,
	parent: Option<BlockId>,
}

impl Block {
	pub fn new(parent: Option<BlockId>) -> Self {
		Self {
			contents: Vec::new(),
			parent,
		}
	}

	pub fn push(&mut self, instr: Instruction) {
		self.contents.push(instr);
	}
}

/// The final result of parsed data
#[derive(Debug)]
pub struct Parsed {
	pub blocks: HashMap<BlockId, Block>,
	pub routines: HashMap<String, BlockId>,
	id_count: BlockId,
}

impl Parsed {
	pub fn new() -> Self {
		let mut out = Self {
			blocks: HashMap::new(),
			routines: HashMap::new(),
			id_count: 0,
		};
		out.routines = HashMap::from([(String::from(DEFAULT_ROUTINE), out.new_block(None))]);
		out
	}

	/// Creates a new block and returns its ID
	pub fn new_block(&mut self, parent: Option<BlockId>) -> BlockId {
		self.id_count += 1;
		self.blocks.insert(self.id_count, Block::new(parent));
		self.id_count
	}

	/// Creates a new routine and its associated block, then returns the block's ID
	pub fn new_routine(&mut self, name: &str) -> BlockId {
		self.new_block(None);
		self.routines.insert(name.to_owned(), self.id_count);
		self.id_count
	}
}

impl Default for Parsed {
	fn default() -> Self {
		Self::new()
	}
}

mod addon {
	use super::*;

	/// State of the addon parser
	#[derive(Debug)]
	pub enum State {
		Id,
		FileName,
		OpenParen,
		Key,
		Colon,
		Value,
		Comma,
		Semicolon,
	}

	/// Current key for the addon parser
	#[derive(Debug)]
	pub enum Key {
		None,
		Kind,
		Url,
		Path,
		Version,
	}

	/// Keys that have been filled
	#[derive(Debug)]
	pub struct FilledKeys {
		pub kind: Option<AddonKind>,
		pub url: Value,
		pub path: Value,
		pub version: Value,
	}
}

pub mod require {
	use super::*;

	/// A single required package
	#[derive(Debug, Clone)]
	pub struct Package {
		pub value: Value,
		pub explicit: bool,
	}

	/// State of the require parser
	#[derive(Debug)]
	pub enum State {
		/// Normal parsing state
		Normal,
	}
}

/// Mode for what we are currently parsing
#[derive(Debug)]
enum ParseMode {
	Root,
	Routine(Option<String>),
	Instruction(Instruction),
	If(Option<Condition>),
	Addon {
		state: addon::State,
		key: addon::Key,
		id: Value,
		file_name: Value,
		filled_keys: addon::FilledKeys,
	},
	Require {
		state: require::State,
		package_groups: Vec<Vec<require::Package>>,
		current_group: Option<Vec<require::Package>>,
		current_package: Option<require::Package>,
		explicit_has_been_closed: bool,
	},
}

#[macro_export]
macro_rules! unexpected_token {
	($tok:expr, $pos:expr) => {
		bail!("Unexpected token {} {}", $tok.as_string(), $pos.clone())
	};
}

/// Data used for parsing
#[derive(Debug)]
struct ParseData {
	parsed: Parsed,
	instruction_n: u32,
	block: BlockId,
	mode: ParseMode,
}

impl ParseData {
	pub fn new() -> Self {
		Self {
			parsed: Parsed::new(),
			instruction_n: 0,
			block: 1,
			mode: ParseMode::Root,
		}
	}

	/// Push a new instruction to the block
	pub fn new_instruction(&mut self, instr: Instruction) {
		self.instruction_n += 1;
		if let Some(block) = self.parsed.blocks.get_mut(&self.block) {
			block.push(instr);
		}
		self.mode = ParseMode::Root;
	}

	/// Finish the current block
	pub fn new_block(&mut self) {
		if let Some(block) = self.parsed.blocks.get_mut(&self.block) {
			if let Some(parent) = block.parent {
				self.block = parent;
			}
		}
	}
}

/// Parse a list of tokens
pub fn parse<'a>(tokens: impl Iterator<Item = &'a TokenAndPos>) -> anyhow::Result<Parsed> {
	let tokens = reduce_tokens(tokens);

	let mut prs = ParseData::new();
	for (tok, pos) in tokens {
		let mut instr_to_push = None;
		let mut mode_to_set = None;
		let mut block_to_set = None;
		let mut block_finished = false;
		match &mut prs.mode {
			ParseMode::Root => {
				match tok {
					Token::At => {
						if let Some(..) = prs
							.parsed
							.blocks
							.get_mut(&prs.block)
							.expect("Block does not exist")
							.parent
						{
							bail!("Unexpected routine {}", pos.clone());
						}
						prs.mode = ParseMode::Routine(None);
					}
					Token::Ident(name) => match name.as_str() {
						"if" => prs.mode = ParseMode::If(None),
						"addon" => {
							prs.mode = ParseMode::Addon {
								state: addon::State::Id,
								key: addon::Key::None,
								id: Value::None,
								file_name: Value::None,
								filled_keys: addon::FilledKeys {
									kind: None,
									url: Value::None,
									path: Value::None,
									version: Value::None,
								},
							};
						}
						"require" => {
							prs.mode = ParseMode::Require {
								state: require::State::Normal,
								package_groups: Vec::new(),
								current_group: None,
								current_package: None,
								explicit_has_been_closed: true,
							};
						}
						name => {
							prs.mode = ParseMode::Instruction(Instruction::from_str(name, pos)?);
						}
					},
					Token::Curly(side) => match side {
						Side::Left => unexpected_token!(tok, pos),
						Side::Right => {
							block_finished = true;
							prs.mode = ParseMode::Root;
						}
					},
					_ => {}
				}
				Ok::<(), anyhow::Error>(())
			}
			ParseMode::Routine(name) => {
				if let Some(name) = name {
					match tok {
						Token::Curly(side) => match side {
							Side::Left => {
								prs.block = prs.parsed.new_routine(name);
								prs.mode = ParseMode::Root;
							}
							Side::Right => unexpected_token!(tok, pos),
						},
						_ => unexpected_token!(tok, pos),
					}
				} else {
					match tok {
						Token::Ident(ident) => {
							*name = Some(ident.to_string());
						}
						_ => unexpected_token!(tok, pos),
					}
				}
				Ok(())
			}
			ParseMode::If(condition) => {
				match tok {
					Token::Curly(Side::Left) => {
						if let Some(condition) = condition {
							if !condition.kind.is_finished_parsing() {
								unexpected_token!(tok, pos);
							}
							let block = prs.parsed.new_block(Some(prs.block));
							block_to_set = Some(block);
							instr_to_push =
								Some(Instruction::new(InstrKind::If(condition.clone(), block)));
							prs.mode = ParseMode::Root;
						}
					}
					Token::Curly(Side::Right) => unexpected_token!(tok, pos),
					_ => match condition {
						Some(condition) => condition.parse(tok, pos)?,
						None => match tok {
							Token::Ident(name) => match ConditionKind::from_str(name) {
								Some(new_condition) => {
									*condition = Some(Condition::new(new_condition))
								}
								None => {
									bail!("Unknown condition {} {}", name.clone(), pos.clone());
								}
							},
							_ => unexpected_token!(tok, pos),
						},
					},
				}

				Ok(())
			}
			ParseMode::Instruction(instr) => {
				if instr.parse(tok, pos)? {
					instr_to_push = Some(instr.clone());
					mode_to_set = Some(ParseMode::Root);
				}

				Ok(())
			}
			ParseMode::Addon {
				state,
				key,
				id,
				file_name,
				filled_keys,
			} => {
				match state {
					addon::State::Id => {
						*id = parse_arg(tok, pos)?;
						*state = addon::State::FileName;
					}
					addon::State::FileName => match tok {
						Token::Paren(Side::Left) => {
							bail!("It is now required to have a filename field for addons");
						}
						_ => {
							*file_name = parse_arg(tok, pos)?;
							*state = addon::State::OpenParen;
						}
					},
					addon::State::OpenParen => match tok {
						Token::Paren(Side::Left) => *state = addon::State::Key,
						_ => unexpected_token!(tok, pos),
					},
					addon::State::Key => match tok {
						Token::Ident(name) => {
							match name.as_str() {
								"kind" => *key = addon::Key::Kind,
								"url" => *key = addon::Key::Url,
								"path" => *key = addon::Key::Path,
								"version" => *key = addon::Key::Version,
								_ => {
									bail!(
										"Unknown key {} for 'addon' instruction {}",
										name.to_owned(),
										pos.clone()
									);
								}
							}
							*state = addon::State::Colon;
						}
						_ => unexpected_token!(tok, pos),
					},
					addon::State::Colon => match tok {
						Token::Colon => *state = addon::State::Value,
						_ => unexpected_token!(tok, pos),
					},
					addon::State::Value => match tok {
						Token::Ident(name) => {
							match key {
								addon::Key::Kind => filled_keys.kind = AddonKind::from_str(name),
								_ => unexpected_token!(tok, &pos),
							}
							*state = addon::State::Comma;
						}
						_ => {
							match key {
								addon::Key::Url => filled_keys.url = parse_arg(tok, pos)?,
								addon::Key::Path => filled_keys.path = parse_arg(tok, pos)?,
								addon::Key::Version => filled_keys.version = parse_arg(tok, pos)?,
								_ => unexpected_token!(tok, pos),
							}
							*state = addon::State::Comma;
						}
					},
					addon::State::Comma => match tok {
						Token::Comma => *state = addon::State::Key,
						Token::Paren(Side::Right) => {
							*state = addon::State::Semicolon;
						}
						_ => unexpected_token!(tok, pos),
					},
					addon::State::Semicolon => match tok {
						Token::Semicolon => {
							instr_to_push = Some(Instruction::new(InstrKind::Addon {
								id: id.clone(),
								file_name: file_name.clone(),
								kind: filled_keys.kind,
								url: filled_keys.url.clone(),
								path: filled_keys.path.clone(),
								version: filled_keys.version.clone(),
							}));
							prs.mode = ParseMode::Root;
						}
						_ => unexpected_token!(tok, pos),
					},
				}

				Ok(())
			}
			ParseMode::Require {
				state,
				package_groups,
				current_group,
				current_package,
				explicit_has_been_closed,
			} => {
				match state {
					require::State::Normal => match tok {
						Token::Paren(Side::Left) => {
							if current_group.is_some() {
								unexpected_token!(tok, pos);
							}
							*current_group = Some(Vec::new());
						}
						Token::Paren(Side::Right) => {
							if current_group.is_none() || !*explicit_has_been_closed {
								unexpected_token!(tok, pos);
							}
							package_groups.extend(current_group.take());
						}
						Token::Angle(Side::Left) => {
							*explicit_has_been_closed = false;
							if current_package.is_some() {
								unexpected_token!(tok, pos);
							}
							*current_package = Some(require::Package {
								value: Value::None,
								explicit: true,
							});
						}
						Token::Angle(Side::Right) => {
							if !*explicit_has_been_closed {
								unexpected_token!(tok, pos);
							}
							*explicit_has_been_closed = true;
						}
						Token::Semicolon => {
							instr_to_push =
								Some(Instruction::new(InstrKind::Require(package_groups.clone())));
							prs.mode = ParseMode::Root;
						}
						_ => {
							let package = parse_arg(tok, pos)?;
							if let Some(current_package) = current_package {
								current_package.value = package;
							} else {
								*current_package = Some(require::Package {
									value: package,
									explicit: false,
								});
							}
							if let Some(group) = current_group {
								group.extend(current_package.take());
							} else {
								let mut vec = Vec::new();
								vec.extend(current_package.take());
								package_groups.push(vec);
							}
							*explicit_has_been_closed = true;
						}
					},
				}

				Ok(())
			}
		}?;

		if let Some(instr) = instr_to_push {
			prs.new_instruction(instr);
		}

		if let Some(mode) = mode_to_set {
			prs.mode = mode;
		}

		if let Some(block) = block_to_set {
			prs.block = block;
		}

		if block_finished {
			prs.new_block();
		}
	}

	Ok(prs.parsed)
}

pub fn lex_and_parse(text: &str) -> anyhow::Result<Parsed> {
	let tokens = lex(text).context("Lexing failed")?;
	let parsed = parse(tokens.iter()).context("Parsing failed")?;
	Ok(parsed)
}

#[cfg(test)]
mod tests {
	use mcvm_shared::modifications::ModloaderMatch;

	use super::*;
	use crate::routine::{INSTALL_ROUTINE, METADATA_ROUTINE};

	#[test]
	fn test_routine_parse() {
		let text = "@install {} @meta {} @foo {}";
		let parsed = lex_and_parse(text).unwrap();
		assert!(parsed
			.blocks
			.contains_key(parsed.routines.get(INSTALL_ROUTINE).unwrap()));
		assert!(parsed
			.blocks
			.contains_key(parsed.routines.get(METADATA_ROUTINE).unwrap()));
		assert!(parsed
			.blocks
			.contains_key(parsed.routines.get("foo").unwrap()));
	}

	#[test]
	fn test_explicit_require_parse() {
		let text = r#"@install { require <"optifine"> <"sodium"> "cit-support"; }"#;
		let parsed = lex_and_parse(text).unwrap();
		let block = parsed
			.blocks
			.get(parsed.routines.get(INSTALL_ROUTINE).unwrap())
			.unwrap();
		for instr in &block.contents {
			if let InstrKind::Require(groups) = &instr.kind {
				let package = groups.get(0).unwrap().get(0).unwrap();
				assert!(matches!(&package.value, Value::Constant(name) if name == "optifine"));
				assert!(package.explicit);

				let package = groups.get(1).unwrap().get(0).unwrap();
				assert!(matches!(&package.value, Value::Constant(name) if name == "sodium"));
				assert!(package.explicit);

				let package = groups.get(2).unwrap().get(0).unwrap();
				assert!(matches!(&package.value, Value::Constant(name) if name == "cit-support"));
				assert!(!package.explicit);
			}
		}
	}

	#[test]
	fn test_and_condition_parse() {
		let text = r#"@install {
			if not modloader fabric and modloader forge {}
		}"#;
		let parsed = lex_and_parse(text).unwrap();
		let block = parsed
			.blocks
			.get(parsed.routines.get(INSTALL_ROUTINE).unwrap())
			.unwrap();
		for instr in &block.contents {
			if let InstrKind::If(condition, _) = &instr.kind {
				assert_eq!(
					condition.kind,
					ConditionKind::And(
						Box::new(ConditionKind::Not(Some(Box::new(
							ConditionKind::Modloader(Some(ModloaderMatch::Fabric))
						)))),
						Some(Box::new(ConditionKind::Modloader(Some(
							ModloaderMatch::Forge
						)))),
					)
				)
			}
		}
	}
}
