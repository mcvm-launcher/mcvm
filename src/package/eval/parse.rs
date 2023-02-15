use super::super::Package;
use super::CommandType;

use trees::{Tree, Node};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
	
}

#[derive(Debug)]
pub enum NodeKind {
	Command(CommandType, Vec<String>),
	Routine(String),
	Root,
	If
}

#[derive(Debug)]
pub struct AstNode {
	pub kind: NodeKind
}

impl AstNode {
	pub fn new(kind: NodeKind) -> Self {
		Self {
			kind
		}
	}
}

pub type PkgAst = Tree<AstNode>;

// Data for the root of an instruction
pub enum ParseRoot {
	Instruction(String, Vec<String>),
	Routine(String),
	None
}

// Modes for what we expect to be parsing
pub enum ParseMode {
	Word(String),
	Str(String)
}

// Data used for parsing
pub struct ParseData {
	// A list of trees for each routine
	ast: PkgAst,
	// The current instruction number
	instruction: u32,
	// Character number in the instruction
	instruction_char: i32,
	// Data for parse root
	root: ParseRoot,
	// Name of the current routine
	routine: Option<String>,
	// Current parsing mode
	mode: ParseMode
}

impl ParseData {
	pub fn new() -> Self {
		Self {
			ast: Tree::new(AstNode::new(NodeKind::Root)),
			instruction: 0,
			instruction_char: -1,
			root: ParseRoot::None,
			routine: None,
			mode: ParseMode::Word(String::new())
		}
	}

	// Reset the instruction data
	pub fn reset_instruction(&mut self) {
		self.instruction += 1;
		self.instruction_char = -1;
		self.root = ParseRoot::None;
	}

	// Parse the current instruction and start a new one
	pub fn new_instruction(&mut self) -> Result<(), ParseError> {
		match &self.root {
			ParseRoot::None => return Ok(()),
			ParseRoot::Routine(name) => {

			}
			ParseRoot::Instruction(name, args) => {

			}
		}
		Ok(())
	}
}

impl Package {
	pub fn parse(&mut self) -> Result<(), ParseError> {
		let mut data = ParseData::new();

		Ok(())
	}
}
