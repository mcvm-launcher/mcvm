use std::fmt::Display;

// Generic side for something like a bracket
#[derive(Debug, PartialEq, Clone)]
pub enum Side {
	Left,
	Right
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
	None,
	Whitespace,
	Semicolon,
	Curly(Side),
	Square(Side),
	Paren(Side),
	Comment(String),
	Ident(String),
	Num(i64),
	Str(String),
	Routine
}

// Text positional information
#[derive(Debug, Clone)]
pub struct TextPos {
	pub row: usize,
	pub col: usize
}

impl Display for TextPos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}:{})", self.row, self.col)
	}
}

#[derive(Debug, thiserror::Error)]
pub enum LexError {
	#[error("Unexpected character '{}' at {}", .0, .1)]
	Unexpected(char, TextPos)
}

#[derive(Debug, PartialEq)]
enum StrLexResult {
	Append,
	Escape,
	End
}

fn lex_string_char(c: char, escape: bool) -> StrLexResult {
	if escape {
		StrLexResult::Append
	} else {
		match c {
			'"' => StrLexResult::End,
			'\\' => StrLexResult::Escape,
			_ => StrLexResult::Append
		}
	}
}

fn is_whitespace(c: char) -> bool {
	c.is_whitespace()
}

fn is_ident(c: char, first: bool) -> bool {
	if first {
		if c.is_numeric() {
			return false;
		}
	}
	c.is_alphanumeric() || c == '_'
}

fn is_num(c: char, first: bool) -> bool {
	if first {
		c.is_numeric() || c == '-'
	} else {
		c.is_numeric()
	}
}

pub fn lex(text: &str) -> Result<Vec<(Token, TextPos)>, LexError> {
	let mut tokens: Vec<(Token, TextPos)> = Vec::new();

	// Positional
	let mut line_n: usize = 0;
	let mut last_line_i: usize = 0;

	// Current token
	let mut tok: Token = Token::None;
	let mut tok_finished = false;

	// Specific token-related vars
	let mut escape = false;
	let mut num_str = String::new();

	for (i, c) in text.chars().enumerate() {
		let pos = TextPos {row: line_n, col: i - last_line_i};
		if c == '\n' {
			line_n += 1;
			last_line_i = i;
		}

		// Using this loop as a goto
		loop {
			let mut repeat = false;
			match &mut tok {
				Token::None => {
					match c {
						';' => {
							tok = Token::Semicolon;
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
						'@' => {
							tok = Token::Routine;
							tok_finished = true;
						}
						'"' => tok = Token::Str(String::new()),
						'#' => tok = Token::Comment(String::new()),
						c if is_whitespace(c) => tok = Token::Whitespace,
						c if is_num(c, true) => {
							tok = Token::Num(0);
							num_str = String::from(c);
						}
						c if is_ident(c, true) => tok = Token::Ident(String::from(c)),
						_ => return Err(LexError::Unexpected(c, pos))
					}
				}
				Token::Str(string) => {
					match lex_string_char(c, escape) {
						StrLexResult::Append => {
							string.push(c);
							escape = false;
						}
						StrLexResult::Escape => escape = true,
						StrLexResult::End => {
							tok_finished = true;
							escape = false;
						}
					}
				}
				Token::Comment(string) => {
					if c == '\n' {
						tok_finished = true;
					} else {
						string.push(c);
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
	let final_pos = TextPos {row: line_n, col: text.len() - last_line_i};
	match &mut tok {
		Token::Num(num) => {
			*num = num_str.parse().expect("Number contains invalid characters");
			tokens.push((tok, final_pos));
		}
		Token::None => {}
		_ => tokens.push((tok, final_pos))
	}
	Ok(tokens)
}

// Removes whitespace characters and comments from a list of tokens
pub fn reduce_tokens(tokens: &[(Token, TextPos)]) -> Vec<(Token, TextPos)> {
	let mut out = Vec::new();
	for (tok, pos) in tokens.iter().cloned() {
		match tok {
			Token::Comment(..) | Token::Whitespace | Token::None => {}
			_ => out.push((tok, pos.clone()))
		}
	}
	out
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
		assert_tokens!(
			";;",
			vec![
				Token::Semicolon,
				Token::Semicolon
			]
		);
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
		assert_tokens!(
			"\"Hello\"",
			vec![
				Token::Str("Hello".to_string())
			]
		);
	}

	#[test]
	fn test_combo() {
		assert_tokens!(
			"\"Uno\"; \"Dos\"; \"Tres\"; Identifier",
			vec![
				Token::Str("Uno".to_string()),
				Token::Semicolon,
				Token::Whitespace,
				Token::Str("Dos".to_string()),
				Token::Semicolon,
				Token::Whitespace,
				Token::Str("Tres".to_string()),
				Token::Semicolon,
				Token::Whitespace,
				Token::Ident("Identifier".to_string())
			]
		);
	}

	#[test]
	fn test_comment() {
		assert_tokens!(
			"\"Foo\" # Comment\n \"Bar\"",
			vec![
				Token::Str("Foo".to_string()),
				Token::Whitespace,
				Token::Comment(" Comment".to_string()),
				Token::Whitespace,
				Token::Str("Bar".to_string())
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
}
