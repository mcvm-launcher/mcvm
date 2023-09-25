use std::fmt::{Debug, Display};

use crate::unexpected_token;
use anyhow::bail;

/// Create a list of tokens from package text contents that we will
/// then use for parsing
pub fn lex(text: &str) -> anyhow::Result<Vec<(Token, TextPos)>> {
	let mut tokens: Vec<(Token, TextPos)> = Vec::new();

	// Positional
	let mut line_n: usize = 1;
	let mut last_line_i: usize = 0;
	let mut tok_start_pos = TextPos(line_n, 0);

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
			// We add one since otherwise the next line starts at column 1 instead of 0
			last_line_i = i + 1;
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
						num_str = c.to_string();
					}
					c if is_ident(c, true) => tok = Token::Ident(c.into()),
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
					let allowed = is_ident(c, name.is_empty());

					if allowed {
						name.push(c);
					} else {
						repeat = true;
						tokens.push((tok, tok_start_pos.clone()));
						tok = Token::None;
					}
				}
				Token::Whitespace => {
					if !is_whitespace(c) {
						repeat = true;
						tokens.push((tok, tok_start_pos.clone()));
						tok_start_pos = pos.clone();
						tok = Token::None;
					}
				}
				Token::Ident(name) => {
					if is_ident(c, false) {
						name.push(c);
					} else {
						repeat = true;
						tokens.push((tok, tok_start_pos.clone()));
						tok_start_pos = pos.clone();
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
						tokens.push((tok, tok_start_pos.clone()));
						tok_start_pos = pos.clone();
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
			tokens.push((tok, tok_start_pos));
			tok_start_pos = pos.clone();
			// Since these are not greedy we need to increase the col by 1
			tok_start_pos.1 += 1;
			tok = Token::None;
		}
	}

	match &mut tok {
		Token::Num(num) => {
			*num = num_str.parse().expect("Number contains invalid characters");
			tokens.push((tok, tok_start_pos.clone()));
		}
		Token::None => {}
		_ => tokens.push((tok, tok_start_pos.clone())),
	}
	Ok(tokens)
}

/// A token that we derive from text
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	/// An empty token with no meaning. These technically shouldn't appear in the output,
	/// but just skip over them.
	None,
	/// Any whitespace, such as tabs and newlines
	Whitespace,
	/// A semicolon (;)
	Semicolon,
	/// A colon (:)
	Colon,
	/// A comma (,)
	Comma,
	/// A pipe / bar (|)
	Pipe,
	/// An at symbol (@)
	At,
	/// An exclamation point (!)
	Bang,
	/// A variable ($var_name)
	Variable(String),
	/// A curly brace ({ / })
	Curly(Side),
	/// A square bracket ([ / ])
	Square(Side),
	/// A parenthese (( / ))
	Paren(Side),
	/// An angle bracket (< / >)
	Angle(Side),
	/// A comment
	Comment(String),
	/// An identifier (foo)
	Ident(String),
	/// An integer number (-12, 6, 128, etc.)
	Num(i64),
	/// A string literal ("'hello' there")
	Str(String),
}

impl Token {
	/// Print this token as a string
	pub fn as_string(&self) -> String {
		match self {
			Token::None => "none".into(),
			Token::Whitespace => " ".into(),
			Token::Semicolon => ";".into(),
			Token::Colon => ":".into(),
			Token::Comma => ",".into(),
			Token::Pipe => "|".into(),
			Token::At => "@".into(),
			Token::Bang => "!".into(),
			Token::Variable(name) => "$".to_string() + name,
			Token::Curly(Side::Left) => "{".into(),
			Token::Curly(Side::Right) => "}".into(),
			Token::Square(Side::Left) => "[".into(),
			Token::Square(Side::Right) => "]".into(),
			Token::Paren(Side::Left) => "(".into(),
			Token::Paren(Side::Right) => ")".into(),
			Token::Angle(Side::Left) => "<".into(),
			Token::Angle(Side::Right) => ">".into(),
			Token::Comment(text) => "# ".to_string() + text,
			Token::Ident(name) => name.clone(),
			Token::Num(num) => num.to_string(),
			Token::Str(string) => format!("\"{string}\""),
		}
	}
}

/// Generic side for something like a bracket
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Side {
	/// Something on the left side (e.g. [)
	Left,
	/// Something on the right side (e.g. ])
	Right,
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

/// Removes whitespace characters and comments from an iterator of tokens
pub fn reduce_tokens<'a, T: Iterator<Item = &'a TokenAndPos>>(
	tokens: T,
) -> impl Iterator<Item = &'a TokenAndPos> {
	tokens.filter(|(tok, ..)| !matches!(tok, Token::Comment(..) | Token::Whitespace | Token::None))
}

#[cfg(test)]
mod tests {
	use super::*;

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
					println!("{e}");
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
		assert_tokens!("\"Hello\"", vec![Token::Str("Hello".into())]);
	}

	#[test]
	fn test_combo() {
		assert_tokens!(
			"\"Uno\"; \"Dos\"; \"Tres\"; Identifier",
			vec![
				Token::Str("Uno".into()),
				Token::Semicolon,
				Token::Whitespace,
				Token::Str("Dos".into()),
				Token::Semicolon,
				Token::Whitespace,
				Token::Str("Tres".into()),
				Token::Semicolon,
				Token::Whitespace,
				Token::Ident("Identifier".into())
			]
		);
	}

	#[test]
	fn test_all() {
		assert_tokens!(
			"\"Hello\"; ident{}@routine[]$var():-1000,|# comment",
			vec![
				Token::Str("Hello".into()),
				Token::Semicolon,
				Token::Whitespace,
				Token::Ident("ident".into()),
				Token::Curly(Side::Left),
				Token::Curly(Side::Right),
				Token::At,
				Token::Ident("routine".into()),
				Token::Square(Side::Left),
				Token::Square(Side::Right),
				Token::Variable("var".into()),
				Token::Paren(Side::Left),
				Token::Paren(Side::Right),
				Token::Colon,
				Token::Num(-1000),
				Token::Comma,
				Token::Pipe,
				Token::Comment(" comment".into())
			]
		);
	}

	#[test]
	fn test_comment() {
		assert_tokens!(
			"\"Foo\" # Comment\n \"Bar\"",
			vec![
				Token::Str("Foo".into()),
				Token::Whitespace,
				Token::Comment(" Comment".into()),
				Token::Whitespace,
				Token::Str("Bar".into())
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

	macro_rules! assert_token_positions {
		($text:literal, $positions:expr) => {
			assert_token_positions!(lex($text), $positions)
		};

		($toks:expr, $positions:expr) => {
			match $toks {
				Ok(toks) => {
					dbg!(&toks, $positions);
					assert_eq!(toks.len(), $positions.len());
					for (i, ((_, tok_pos), (expected_row, expected_col))) in
						toks.iter().zip($positions.iter()).enumerate()
					{
						let expected_pos = TextPos(*expected_row, *expected_col);
						assert_eq!(tok_pos, &expected_pos, "Index: {i}");
					}
				}
				Err(e) => {
					println!("{e}");
					panic!();
				}
			}
		};
	}

	#[test]
	fn test_token_pos_simple() {
		assert_token_positions!(
			"hello;world!!\nwhy\nwhere",
			[
				(1, 0),
				(1, 5),
				(1, 6),
				(1, 11),
				(1, 12),
				(1, 13),
				(2, 0),
				(2, 3),
				(3, 0)
			]
		);
	}
}
