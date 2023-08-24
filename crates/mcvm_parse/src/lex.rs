use std::fmt::{Debug, Display};

use crate::unexpected_token;
use anyhow::bail;

/// Generic side for something like a bracket
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Side {
	Left,
	Right,
}

/// A token that we derive from text
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	None,
	Whitespace,
	Semicolon,
	Colon,
	Comma,
	Pipe,
	At,
	Bang,
	Variable(String),
	Curly(Side),
	Square(Side),
	Paren(Side),
	Angle(Side),
	Comment(String),
	Ident(String),
	Num(i64),
	Str(String),
}

impl Token {
	pub fn as_string(&self) -> String {
		match self {
			Token::None => String::from("none"),
			Token::Whitespace => String::from(" "),
			Token::Semicolon => String::from(";"),
			Token::Colon => String::from(":"),
			Token::Comma => String::from(","),
			Token::Pipe => String::from("|"),
			Token::At => String::from("@"),
			Token::Bang => String::from("!"),
			Token::Variable(name) => String::from("$") + name,
			Token::Curly(Side::Left) => String::from("{"),
			Token::Curly(Side::Right) => String::from("}"),
			Token::Square(Side::Left) => String::from("["),
			Token::Square(Side::Right) => String::from("]"),
			Token::Paren(Side::Left) => String::from("("),
			Token::Paren(Side::Right) => String::from(")"),
			Token::Angle(Side::Left) => String::from("<"),
			Token::Angle(Side::Right) => String::from(">"),
			Token::Comment(text) => String::from("# ") + text,
			Token::Ident(name) => name.clone(),
			Token::Num(num) => num.to_string(),
			Token::Str(string) => format!("\"{string}\""),
		}
	}
}

/// Text positional information with row and column
#[derive(Clone, PartialEq, Eq)]
pub struct TextPos(usize, usize);

impl Debug for TextPos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}:{})", self.0, self.1)
	}
}

impl Display for TextPos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}:{})", self.0, self.1)
	}
}

/// Token and TextPos
pub type TokenAndPos = (Token, TextPos);

/// What action to perform after lexing a string character
#[derive(Debug, PartialEq)]
enum StrLexResult {
	Append,
	Escape,
	End,
}

/// Figure out what to do with a character of a string when lexing
fn lex_string_char(c: char, escape: bool) -> StrLexResult {
	if escape {
		StrLexResult::Append
	} else {
		match c {
			'"' => StrLexResult::End,
			'\\' => StrLexResult::Escape,
			_ => StrLexResult::Append,
		}
	}
}

fn is_whitespace(c: char) -> bool {
	c.is_whitespace()
}

/// Checks if a character is part of a valid identifier
fn is_ident(c: char, first: bool) -> bool {
	if first && c.is_numeric() {
		return false;
	}
	c.is_alphanumeric() || c == '_'
}

/// Checks if a character is part of an integer
fn is_num(c: char, first: bool) -> bool {
	if first {
		c.is_numeric() || c == '-'
	} else {
		c.is_numeric()
	}
}

/// Create a list of tokens from package text contents that we will
/// then use for parsing
pub fn lex(text: &str) -> anyhow::Result<Vec<(Token, TextPos)>> {
	let mut tokens: Vec<(Token, TextPos)> = Vec::new();

	// Positional
	let mut line_n: usize = 1;
	let mut last_line_i: usize = 0;

	// Current token
	let mut tok: Token = Token::None;
	let mut tok_finished = false;

	// Specific token-related vars
	let mut escape = false;
	let mut num_str = String::new();

	for (i, c) in text.chars().enumerate() {
		let pos = TextPos(line_n, i - last_line_i);
		if c == '\n' {
			line_n += 1;
			last_line_i = i;
		}

		// Using this loop as a goto
		loop {
			let mut repeat = false;
			match &mut tok {
				Token::None => match c {
					';' => {
						tok = Token::Semicolon;
						tok_finished = true;
					}
					':' => {
						tok = Token::Colon;
						tok_finished = true;
					}
					',' => {
						tok = Token::Comma;
						tok_finished = true;
					}
					'|' => {
						tok = Token::Pipe;
						tok_finished = true;
					}
					'{' => {
						tok = Token::Curly(Side::Left);
						tok_finished = true;
					}
					'}' => {
						tok = Token::Curly(Side::Right);
						tok_finished = true;
					}
					'[' => {
						tok = Token::Square(Side::Left);
						tok_finished = true;
					}
					']' => {
						tok = Token::Square(Side::Right);
						tok_finished = true;
					}
					'(' => {
						tok = Token::Paren(Side::Left);
						tok_finished = true;
					}
					')' => {
						tok = Token::Paren(Side::Right);
						tok_finished = true;
					}
					'<' => {
						tok = Token::Angle(Side::Left);
						tok_finished = true;
					}
					'>' => {
						tok = Token::Angle(Side::Right);
						tok_finished = true;
					}
					'@' => {
						tok = Token::At;
						tok_finished = true;
					}
					'!' => {
						tok = Token::Bang;
						tok_finished = true;
					}
					'"' => tok = Token::Str(String::new()),
					'#' => tok = Token::Comment(String::new()),
					'$' => tok = Token::Variable(String::new()),
					c if is_whitespace(c) => tok = Token::Whitespace,
					c if is_num(c, true) => {
						tok = Token::Num(0);
						num_str = String::from(c);
					}
					c if is_ident(c, true) => tok = Token::Ident(String::from(c)),
					_ => unexpected_token!(tok, pos),
				},
				Token::Str(string) => match lex_string_char(c, escape) {
					StrLexResult::Append => {
						string.push(c);
						escape = false;
					}
					StrLexResult::Escape => escape = true,
					StrLexResult::End => {
						tok_finished = true;
						escape = false;
					}
				},
				Token::Comment(string) => {
					if c == '\n' {
						tok_finished = true;
					} else {
						string.push(c);
					}
				}
				Token::Variable(name) => {
					let allowed = if name.is_empty() {
						is_ident(c, true)
					} else {
						is_ident(c, false)
					};

					if allowed {
						name.push(c);
					} else {
						repeat = true;
						tokens.push((tok, pos.clone()));
						tok = Token::None;
					}
				}
				Token::Whitespace => {
					if !is_whitespace(c) {
						repeat = true;
						tokens.push((tok, pos.clone()));
						tok = Token::None;
					}
				}
				Token::Ident(name) => {
					if is_ident(c, false) {
						name.push(c);
					} else {
						repeat = true;
						tokens.push((tok, pos.clone()));
						tok = Token::None;
					}
				}
				Token::Num(num) => {
					if is_num(c, false) {
						num_str.push(c);
					} else {
						repeat = true;
						if num_str == "-" {
							bail!("Invalid number '{num_str}', {pos}");
						}
						*num = num_str.parse().expect("Number contains invalid characters");
						tokens.push((tok, pos.clone()));
						tok = Token::None;
					}
				}
				_ => {}
			}
			if !repeat {
				break;
			}
		}
		if tok_finished {
			tok_finished = false;
			tokens.push((tok, pos));
			tok = Token::None;
		}
	}
	let final_pos = TextPos(line_n, text.len() - last_line_i);
	match &mut tok {
		Token::Num(num) => {
			*num = num_str.parse().expect("Number contains invalid characters");
			tokens.push((tok, final_pos));
		}
		Token::None => {}
		_ => tokens.push((tok, final_pos)),
	}
	Ok(tokens)
}

/// Removes whitespace characters and comments from an iterator of tokens
pub fn reduce_tokens<'a, T: Iterator<Item = &'a TokenAndPos>>(
	tokens: T,
) -> impl Iterator<Item = &'a TokenAndPos> {
	tokens.filter(|(tok, ..)| !matches!(tok, Token::Comment(..) | Token::Whitespace | Token::None))
}

#[cfg(test)]
mod tests {
	use super::*;
	use color_print::cprintln;

	macro_rules! assert_tokens {
		($text:literal, $toks:expr) => {
			assert_tokens!(lex($text), $toks)
		};

		($lexed:expr, $toks:expr) => {
			match $lexed {
				Ok(lexed) => {
					assert_eq!(lexed.len(), $toks.len());
					for ((left, _), right) in lexed.iter().zip($toks) {
						assert_eq!(left, &right);
					}
				}
				Err(e) => {
					cprintln!("<r>{}", e);
					panic!();
				}
			};
		};
	}

	#[test]
	fn test_chars() {
		assert!(is_ident('a', false));
		assert!(is_ident('a', true));
		assert!(is_ident('B', false));
		assert!(is_ident('B', true));
		assert!(is_ident('_', false));
		assert!(is_ident('_', true));

		assert!(is_ident('5', false));
		assert!(!is_ident('2', true));

		assert!(is_num('8', false));
		assert!(is_num('8', true));
		assert!(!is_num('t', false));
		assert!(!is_num('t', true));
		assert!(!is_num('.', false));
		assert!(!is_num('.', true));
		assert!(is_num('-', true));
		assert!(!is_num('-', false));

		assert!(is_whitespace(' '));
		assert!(is_whitespace('\n'));
		assert!(!is_whitespace('a'));
		assert!(!is_whitespace('%'));
	}

	#[test]
	fn test_semicolon() {
		assert_tokens!(";;", vec![Token::Semicolon, Token::Semicolon]);
	}

	#[test]
	fn test_string_chars() {
		assert_eq!(lex_string_char('d', false), StrLexResult::Append);
		assert_eq!(lex_string_char('\'', false), StrLexResult::Append);
		assert_eq!(lex_string_char('"', false), StrLexResult::End);
		assert_eq!(lex_string_char('"', true), StrLexResult::Append);
		assert_eq!(lex_string_char('\\', false), StrLexResult::Escape);
		assert_eq!(lex_string_char('\\', true), StrLexResult::Append);
	}

	#[test]
	fn test_string() {
		assert_tokens!("\"Hello\"", vec![Token::Str(String::from("Hello"))]);
	}

	#[test]
	fn test_combo() {
		assert_tokens!(
			"\"Uno\"; \"Dos\"; \"Tres\"; Identifier",
			vec![
				Token::Str(String::from("Uno")),
				Token::Semicolon,
				Token::Whitespace,
				Token::Str(String::from("Dos")),
				Token::Semicolon,
				Token::Whitespace,
				Token::Str(String::from("Tres")),
				Token::Semicolon,
				Token::Whitespace,
				Token::Ident(String::from("Identifier"))
			]
		);
	}

	#[test]
	fn test_all() {
		assert_tokens!(
			"\"Hello\"; ident{}@routine[]$var():-1000,|# comment",
			vec![
				Token::Str(String::from("Hello")),
				Token::Semicolon,
				Token::Whitespace,
				Token::Ident(String::from("ident")),
				Token::Curly(Side::Left),
				Token::Curly(Side::Right),
				Token::At,
				Token::Ident(String::from("routine")),
				Token::Square(Side::Left),
				Token::Square(Side::Right),
				Token::Variable(String::from("var")),
				Token::Paren(Side::Left),
				Token::Paren(Side::Right),
				Token::Colon,
				Token::Num(-1000),
				Token::Comma,
				Token::Pipe,
				Token::Comment(String::from(" comment"))
			]
		);
	}

	#[test]
	fn test_comment() {
		assert_tokens!(
			"\"Foo\" # Comment\n \"Bar\"",
			vec![
				Token::Str(String::from("Foo")),
				Token::Whitespace,
				Token::Comment(String::from(" Comment")),
				Token::Whitespace,
				Token::Str(String::from("Bar"))
			]
		);
	}

	#[test]
	fn test_num() {
		assert_tokens!(
			"12345;888;0;-10",
			vec![
				Token::Num(12345),
				Token::Semicolon,
				Token::Num(888),
				Token::Semicolon,
				Token::Num(0),
				Token::Semicolon,
				Token::Num(-10)
			]
		);
	}

	macro_rules! _assert_token_positions {
		($text:literal, $positions:expr) => {
			assert_token_positions!(lex($text), $positions)
		};

		($toks:expr, $positions:expr) => {
			match $toks {
				Ok(toks) => {
					dbg!(&toks, $positions);
					assert_eq!(toks.len(), $positions.len());
					for (i, ((_, tok_pos), expected_pos)) in
						toks.iter().zip($positions.iter()).enumerate()
					{
						assert_eq!(tok_pos, expected_pos, "Index: {i}");
					}
				}
				Err(e) => {
					cprintln!("<r>{}", e);
					panic!();
				}
			}
		};
	}
}
