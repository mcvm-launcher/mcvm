use anyhow::anyhow;
use anyhow::{bail, Context};
use mcvm_shared::pkg::PackageAddonHashes;

use crate::instruction::ElseBlock;
use crate::routine::can_call_routines;
use crate::routine::RESERVED_ROUTINES;

use super::conditions::Condition;
use super::conditions::ConditionKind;
use super::instruction::{parse_arg, InstrKind, Instruction};
use super::lex::{lex, reduce_tokens, Side, Token, TokenAndPos};
use super::vars::Value;
use mcvm_shared::addon::AddonKind;

use std::collections::{HashMap, VecDeque};

const DEFAULT_ROUTINE: &str = "__default__";

/// Throw an anyhow error about an unexpected token at a position
#[macro_export]
macro_rules! unexpected_token {
	($tok:expr, $pos:expr) => {
		bail!("Unexpected token {} {}", $tok.as_string(), $pos.clone())
	};
}

/// Parse a list of tokens
pub fn parse<'a>(tokens: impl Iterator<Item = &'a TokenAndPos>) -> anyhow::Result<Parsed> {
	let tokens = reduce_tokens(tokens);

	let mut prs = ParseData::new();
	// Whether or not a block just ended
	let mut block_just_ended = false;
	for (tok, pos) in tokens {
		let mut instr_to_push = None;
		let mut mode_to_set = None;
		let mut block_to_set = None;
		let mut block_finished = false;
		let mut else_to_append = None;
		match &mut prs.mode {
			ParseMode::Root => {
				match tok {
					Token::At => {
						if prs
							.parsed
							.blocks
							.get_mut(&prs.block)
							.expect("Block does not exist")
							.parent
							.is_some()
						{
							bail!("Unexpected routine {}", pos.clone());
						}
						prs.mode = ParseMode::Routine(None);
					}
					Token::Ident(name) => match name.as_str() {
						"if" => {
							prs.mode = ParseMode::If {
								condition: None,
								is_if_else: false,
							};
							block_just_ended = false;
						}
						"else" => {
							if !block_just_ended {
								bail!("'else' used without if block {}", pos.clone());
							}
							prs.mode = ParseMode::CheckForElseIf;
							block_just_ended = false;
						}
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
									hashes: PackageAddonHashes {
										sha256: Value::None,
										sha512: Value::None,
									},
								},
							};
							block_just_ended = false;
						}
						"require" => {
							prs.mode = ParseMode::Require {
								state: require::State::Normal,
								package_groups: Vec::new(),
								current_group: None,
								current_package: None,
								explicit_has_been_closed: true,
							};
							block_just_ended = false;
						}
						name => {
							prs.mode =
								ParseMode::Instruction(Instruction::from_str(name, pos.clone())?);
							block_just_ended = false;
						}
					},
					Token::Curly(side) => match side {
						Side::Left => unexpected_token!(tok, pos),
						Side::Right => {
							block_finished = true;
							block_just_ended = true;
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
								if prs.parsed.routine_exists(name) {
									bail!("Redefinition of routine '{name}' {pos}");
								}
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
			ParseMode::If {
				condition,
				is_if_else,
			} => {
				match tok {
					Token::Curly(Side::Left) => {
						if let Some(condition) = condition {
							if !condition.kind.is_finished_parsing() {
								unexpected_token!(tok, pos);
							}
							let block = prs.parsed.new_block(Some(prs.block));
							if *is_if_else {
								else_to_append = Some(ElseBlock {
									block,
									condition: Some(condition.clone()),
								});
							} else {
								block_to_set = Some(block);
								instr_to_push = Some(Instruction::new(
									InstrKind::If {
										condition: condition.clone(),
										if_block: block,
										else_blocks: Vec::new(),
									},
									pos.clone(),
								));
							}
							prs.mode = ParseMode::Root;
						} else {
							unexpected_token!(tok, pos);
						}
					}
					Token::Curly(Side::Right) => unexpected_token!(tok, pos),
					_ => match condition {
						Some(condition) => condition.parse(tok, pos)?,
						None => match tok {
							Token::Ident(name) => match ConditionKind::parse_from_str(name) {
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
			ParseMode::CheckForElseIf => {
				match tok {
					Token::Ident(name) => match name.as_str() {
						// Start an else if
						"if" => {
							prs.mode = ParseMode::If {
								condition: None,
								is_if_else: true,
							};
						}
						_ => unexpected_token!(tok, pos),
					},
					// No condition, just append the else block
					Token::Curly(Side::Left) => {
						let block = prs.parsed.new_block(Some(prs.block));
						else_to_append = Some(ElseBlock {
							block,
							condition: None,
						});
						prs.mode = ParseMode::Root;
					}
					_ => unexpected_token!(tok, pos),
				}

				Ok(())
			}
			ParseMode::Instruction(instr) => {
				if instr
					.parse(tok, pos)
					.with_context(|| format!("Failed to parse instruction {instr} {pos}"))?
				{
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
						Token::Paren(Side::Left) => *state = addon::State::Key,
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
								"hash_sha256" => *key = addon::Key::HashSHA256,
								"hash_sha512" => *key = addon::Key::HashSHA512,
								_ => {
									bail!(
										"Unknown key {} for 'addon' instruction {}",
										name.to_string(),
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
								addon::Key::Kind => {
									filled_keys.kind = AddonKind::parse_from_str(name)
								}
								_ => unexpected_token!(tok, &pos),
							}
							*state = addon::State::Comma;
						}
						_ => {
							let arg = parse_arg(tok, pos)?;
							match key {
								addon::Key::Url => filled_keys.url = arg,
								addon::Key::Path => filled_keys.path = arg,
								addon::Key::Version => filled_keys.version = arg,
								addon::Key::HashSHA256 => filled_keys.hashes.sha256 = arg,
								addon::Key::HashSHA512 => filled_keys.hashes.sha512 = arg,
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
							instr_to_push = Some(Instruction::new(
								InstrKind::Addon {
									id: id.clone(),
									file_name: file_name.clone(),
									kind: filled_keys.kind,
									url: filled_keys.url.clone(),
									path: filled_keys.path.clone(),
									version: filled_keys.version.clone(),
									hashes: filled_keys.hashes.clone(),
								},
								pos.clone(),
							));
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
							instr_to_push = Some(Instruction::new(
								InstrKind::Require(package_groups.clone()),
								pos.clone(),
							));
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

		if let Some(else_to_append) = else_to_append {
			let block = else_to_append.block;
			// We append this to the last instruction, which we assume is an if
			let last_instr = prs
				.last_instruction()
				.ok_or(anyhow!("Else was not used after if block {}", pos))?;
			if let InstrKind::If { else_blocks, .. } = &mut last_instr.kind {
				else_blocks.push(else_to_append);
			} else {
				bail!("Else was not used after if block {}", pos);
			}

			block_to_set = Some(block);
		}

		if let Some(block) = block_to_set {
			prs.block = block;
		}

		if block_finished {
			prs.finish_block();
		}
	}

	// Check for recursion
	check_recursion(&prs.parsed)?;

	Ok(prs.parsed)
}

mod addon {
	use mcvm_shared::pkg::PackageAddonHashes;

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
		HashSHA256,
		HashSHA512,
	}

	/// Keys that have been filled
	#[derive(Debug)]
	pub struct FilledKeys {
		pub kind: Option<AddonKind>,
		pub url: Value,
		pub path: Value,
		pub version: Value,
		pub hashes: PackageAddonHashes<Value>,
	}
}

/// Data for parsing the require instruction
pub mod require {
	use super::*;

	/// A single required package
	#[derive(Debug, Clone)]
	pub struct Package {
		/// The package ID that is required
		pub value: Value,
		/// Whether or not this is an explicit dependency
		pub explicit: bool,
	}

	/// State of the require parser
	#[derive(Debug)]
	pub enum State {
		/// Normal parsing state
		Normal,
	}
}

/// Data used for parsing
#[derive(Debug)]
struct ParseData {
	parsed: Parsed,
	instruction_n: u32,
	block: BlockId,
	mode: ParseMode,
}

/// Mode for what we are currently parsing
#[derive(Debug)]
enum ParseMode {
	Root,
	Routine(Option<String>),
	Instruction(Instruction),
	If {
		condition: Option<Condition>,
		is_if_else: bool,
	},
	CheckForElseIf,
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
	pub fn finish_block(&mut self) {
		if let Some(block) = self.parsed.blocks.get_mut(&self.block) {
			if let Some(parent) = block.parent {
				self.block = parent;
			}
		}
	}

	/// Get the last instruction in the current block
	pub fn last_instruction(&mut self) -> Option<&mut Instruction> {
		let block = self.parsed.blocks.get_mut(&self.block)?;
		block.contents.last_mut()
	}
}

/// The final result of parsed data
#[derive(Debug)]
pub struct Parsed {
	/// The blocks of instructions that have been parsed
	pub blocks: HashMap<BlockId, Block>,
	/// A map of routine names to the blocks they contain
	pub routines: HashMap<String, BlockId>,
	id_count: BlockId,
}

impl Parsed {
	/// Create a new Parsed
	pub fn new() -> Self {
		let mut out = Self {
			blocks: HashMap::new(),
			routines: HashMap::new(),
			id_count: 0,
		};
		out.routines = HashMap::from([(DEFAULT_ROUTINE.into(), out.new_block(None))]);
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
		self.routines.insert(name.to_string(), self.id_count);
		self.id_count
	}

	/// Checks if a routine exists
	pub fn routine_exists(&self, name: &str) -> bool {
		self.routines.contains_key(name)
	}
}

impl Default for Parsed {
	fn default() -> Self {
		Self::new()
	}
}

/// The type we use to index blocks in the hashmap
pub type BlockId = u16;

/// A list of instructions inside a routine or nested block (such as an if block)
#[derive(Debug, Clone)]
pub struct Block {
	/// The instructions contained in the block, in order
	pub contents: Vec<Instruction>,
	parent: Option<BlockId>,
}

impl Block {
	/// Create a new block with an optional parent block that is used to return context when parsing
	pub fn new(parent: Option<BlockId>) -> Self {
		Self {
			contents: Vec::new(),
			parent,
		}
	}

	/// Add an instruction to the block
	pub fn push(&mut self, instr: Instruction) {
		self.contents.push(instr);
	}
}

/// Checks a Parsed for recursion
fn check_recursion(parsed: &Parsed) -> anyhow::Result<()> {
	/// Recursive function to check a routine for recursion
	fn check_routine(
		parsed: &Parsed,
		routine: &str,
		stack: &mut VecDeque<String>,
	) -> anyhow::Result<()> {
		let routine_id = parsed
			.routines
			.get(routine)
			.ok_or(anyhow!("Routine '{routine}' does not exist"))?;
		let block = parsed.blocks.get(routine_id).expect("Block does not exist");
		check_block(parsed, routine, block, stack)?;

		Ok(())
	}

	// Check a block. Separated out due to if blocks and such
	fn check_block(
		parsed: &Parsed,
		parent_routine: &str,
		block: &Block,
		stack: &mut VecDeque<String>,
	) -> anyhow::Result<()> {
		for instr in &block.contents {
			match &instr.kind {
				InstrKind::Call(target) => {
					let target = target.get();
					if stack.contains(target) {
						bail!("Recursion detected calling routine '{target}'");
					}

					stack.push_back(parent_routine.to_string());

					check_routine(parsed, target, stack)
						.with_context(|| format!("From routine '{parent_routine}'"))?;

					let popped = stack.pop_back();
					assert_eq!(popped, Some(parent_routine.to_string()));
				}
				InstrKind::If {
					if_block,
					else_blocks,
					..
				} => {
					let if_block = parsed
						.blocks
						.get(if_block)
						.expect("If block does not exist");
					check_block(parsed, parent_routine, if_block, stack)?;
					for else_block in else_blocks {
						let else_block = parsed
							.blocks
							.get(&else_block.block)
							.expect("If else block does not exist");
						check_block(parsed, parent_routine, else_block, stack)?;
					}
				}
				_ => {}
			}
		}

		Ok(())
	}

	for reserved_routine in RESERVED_ROUTINES {
		if can_call_routines(reserved_routine) && parsed.routine_exists(reserved_routine) {
			let mut stack = VecDeque::new();
			check_routine(parsed, reserved_routine, &mut stack)?;
		}
	}

	Ok(())
}

/// Lex text into tokens and then parse the result
pub fn lex_and_parse(text: &str) -> anyhow::Result<Parsed> {
	let tokens = lex(text).context("Lexing failed")?;
	let parsed = parse(tokens.iter()).context("Parsing failed")?;
	Ok(parsed)
}

#[cfg(test)]
mod tests {
	use mcvm_shared::{later::Later, modifications::ModloaderMatch};

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
				assert!(matches!(&package.value, Value::Literal(name) if name == "optifine"));
				assert!(package.explicit);

				let package = groups.get(1).unwrap().get(0).unwrap();
				assert!(matches!(&package.value, Value::Literal(name) if name == "sodium"));
				assert!(package.explicit);

				let package = groups.get(2).unwrap().get(0).unwrap();
				assert!(matches!(&package.value, Value::Literal(name) if name == "cit-support"));
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
			if let InstrKind::If { condition, .. } = &instr.kind {
				assert_eq!(
					condition.kind,
					ConditionKind::And(
						Box::new(ConditionKind::Not(Later::Full(Box::new(
							ConditionKind::Modloader(Later::Full(ModloaderMatch::Fabric))
						)))),
						Later::Full(Box::new(ConditionKind::Modloader(Later::Full(
							ModloaderMatch::Forge
						)))),
					)
				)
			}
		}
	}

	#[test]
	fn test_if_else() {
		let text = r#"@install {
			if value "" "" {
				finish;
			} else if value "foo" "bar" {
				set x "";
			} else {
				set y "";
			}
			set z "";
		}"#;
		let parsed = lex_and_parse(text).unwrap();
		let block = parsed
			.blocks
			.get(parsed.routines.get(INSTALL_ROUTINE).unwrap())
			.unwrap();
		for instr in &block.contents {
			if let InstrKind::If {
				condition,
				else_blocks,
				..
			} = &instr.kind
			{
				assert_eq!(
					condition.kind,
					ConditionKind::Value(
						Value::Literal(String::new()),
						Value::Literal(String::new())
					)
				);
				let else_block = else_blocks.get(0).unwrap();
				assert_eq!(
					else_block.condition.clone().unwrap().kind,
					ConditionKind::Value(
						Value::Literal("foo".into()),
						Value::Literal("bar".into())
					)
				);
				let else_block = else_blocks.get(1).unwrap();
				assert!(else_block.condition.is_none());
			}
		}
	}

	#[test]
	#[should_panic]
	fn test_no_duplicate_routines() {
		let text = r#"@install {} @install {}"#;
		lex_and_parse(text).unwrap();
	}

	#[test]
	#[should_panic]
	fn test_recursion_checking() {
		let text = r#"
			@a {
				call c;
			}
			@b {
				if defined foo {
					call a;
				}
			}
			@c {
				call b;
			}
			@d {}

			@install {
				call d;
				call c;
			}
		"#;

		lex_and_parse(text).unwrap();
	}

	#[test]
	fn test_addon_parse() {
		let text = r#"@install { addon "mod" "H.jar" (kind: mod); addon "pack" (kind: mod); }"#;
		lex_and_parse(text).unwrap();
	}
}
