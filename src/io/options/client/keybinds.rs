use std::fmt::Display;

use serde::Deserialize;

use crate::util::ToInt;

/// Version-agnostic keybinds
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Keybind {
	Unbound,
	MouseLeft,
	MouseRight,
	MouseMiddle,
	Mouse4,
	Mouse5,
	Mouse6,
	Mouse7,
	Mouse8,
	Num0,
	Num1,
	Num2,
	Num3,
	Num4,
	Num5,
	Num6,
	Num7,
	Num8,
	Num9,
	A,
	B,
	C,
	D,
	E,
	F,
	G,
	H,
	I,
	J,
	K,
	L,
	M,
	N,
	O,
	P,
	Q,
	R,
	S,
	T,
	U,
	V,
	W,
	X,
	Y,
	Z,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	F10,
	F11,
	F12,
	F13,
	F14,
	F15,
	F16,
	F17,
	F18,
	F19,
	F20,
	F21,
	F22,
	F23,
	F24,
	F25,
	NumLock,
	Numpad0,
	Numpad1,
	Numpad2,
	Numpad3,
	Numpad4,
	Numpad5,
	Numpad6,
	Numpad7,
	Numpad8,
	Numpad9,
	NumpadAdd,
	NumpadDecimal,
	NumpadEnter,
	NumpadEqual,
	NumpadMultiply,
	NumpadDivide,
	NumpadSubtract,
	Down,
	Left,
	Right,
	Up,
	Apostrophe,
	Backslash,
	Comma,
	Equal,
	GraveAccent,
	LeftBracket,
	RightBracket,
	Minus,
	Period,
	Semicolon,
	Slash,
	Space,
	Tab,
	LeftAlt,
	RightAlt,
	LeftShift,
	RightShift,
	LeftControl,
	RightControl,
	LeftSystem,
	RightSystem,
	Enter,
	Escape,
	Backspace,
	Delete,
	Home,
	End,
	Insert,
	PageDown,
	PageUp,
	CapsLock,
	Pause,
	ScrollLock,
	Menu,
	PrintScreen,
	World1,
	World2,
}

impl Display for Keybind {
	/// Convert to the keycode used after 1.13
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::Unbound => "key.keyboard.unknown",
				Self::MouseLeft => "key.mouse.left",
				Self::MouseRight => "key.mouse.right",
				Self::MouseMiddle => "key.mouse.middle",
				Self::Mouse4 => "key.mouse.4",
				Self::Mouse5 => "key.mouse.5",
				Self::Mouse6 => "key.mouse.6",
				Self::Mouse7 => "key.mouse.7",
				Self::Mouse8 => "key.mouse.8",
				Self::Num0 => "key.keyboard.0",
				Self::Num1 => "key.keyboard.1",
				Self::Num2 => "key.keyboard.2",
				Self::Num3 => "key.keyboard.3",
				Self::Num4 => "key.keyboard.4",
				Self::Num5 => "key.keyboard.5",
				Self::Num6 => "key.keyboard.6",
				Self::Num7 => "key.keyboard.7",
				Self::Num8 => "key.keyboard.8",
				Self::Num9 => "key.keyboard.9",
				Self::A => "key.keyboard.a",
				Self::B => "key.keyboard.b",
				Self::C => "key.keyboard.c",
				Self::D => "key.keyboard.d",
				Self::E => "key.keyboard.e",
				Self::F => "key.keyboard.f",
				Self::G => "key.keyboard.g",
				Self::H => "key.keyboard.h",
				Self::I => "key.keyboard.i",
				Self::J => "key.keyboard.j",
				Self::K => "key.keyboard.k",
				Self::L => "key.keyboard.l",
				Self::M => "key.keyboard.m",
				Self::N => "key.keyboard.n",
				Self::O => "key.keyboard.o",
				Self::P => "key.keyboard.p",
				Self::Q => "key.keyboard.q",
				Self::R => "key.keyboard.r",
				Self::S => "key.keyboard.s",
				Self::T => "key.keyboard.t",
				Self::U => "key.keyboard.u",
				Self::V => "key.keyboard.v",
				Self::W => "key.keyboard.w",
				Self::X => "key.keyboard.x",
				Self::Y => "key.keyboard.y",
				Self::Z => "key.keyboard.z",
				Self::F1 => "key.keyboard.f1",
				Self::F2 => "key.keyboard.f2",
				Self::F3 => "key.keyboard.f3",
				Self::F4 => "key.keyboard.f4",
				Self::F5 => "key.keyboard.f5",
				Self::F6 => "key.keyboard.f6",
				Self::F7 => "key.keyboard.f7",
				Self::F8 => "key.keyboard.f8",
				Self::F9 => "key.keyboard.f9",
				Self::F10 => "key.keyboard.f10",
				Self::F11 => "key.keyboard.f11",
				Self::F12 => "key.keyboard.f12",
				Self::F13 => "key.keyboard.f13",
				Self::F14 => "key.keyboard.f14",
				Self::F15 => "key.keyboard.f15",
				Self::F16 => "key.keyboard.f16",
				Self::F17 => "key.keyboard.f17",
				Self::F18 => "key.keyboard.f18",
				Self::F19 => "key.keyboard.f19",
				Self::F20 => "key.keyboard.f20",
				Self::F21 => "key.keyboard.f21",
				Self::F22 => "key.keyboard.f22",
				Self::F23 => "key.keyboard.f23",
				Self::F24 => "key.keyboard.f24",
				Self::F25 => "key.keyboard.f25",
				Self::NumLock => "key.keyboard.num.lock",
				Self::Numpad0 => "key.keyboard.keypad.0",
				Self::Numpad1 => "key.keyboard.keypad.1",
				Self::Numpad2 => "key.keyboard.keypad.2",
				Self::Numpad3 => "key.keyboard.keypad.3",
				Self::Numpad4 => "key.keyboard.keypad.4",
				Self::Numpad5 => "key.keyboard.keypad.5",
				Self::Numpad6 => "key.keyboard.keypad.6",
				Self::Numpad7 => "key.keyboard.keypad.7",
				Self::Numpad8 => "key.keyboard.keypad.8",
				Self::Numpad9 => "key.keyboard.keypad.9",
				Self::NumpadAdd => "key.keyboard.keypad.add",
				Self::NumpadDecimal => "key.keyboard.keypad.decimal",
				Self::NumpadEnter => "key.keyboard.keypad.enter",
				Self::NumpadEqual => "key.keyboard.keypad.equal",
				Self::NumpadMultiply => "key.keyboard.keypad.multiply",
				Self::NumpadDivide => "key.keyboard.keypad.divide",
				Self::NumpadSubtract => "key.keyboard.keypad.subtract",
				Self::Down => "key.keyboard.down",
				Self::Left => "key.keyboard.left",
				Self::Right => "key.keyboard.right",
				Self::Up => "key.keyboard.up",
				Self::Apostrophe => "key.keyboard.apostrophe",
				Self::Backslash => "key.keyboard.backslash",
				Self::Comma => "key.keyboard.comma",
				Self::Equal => "key.keyboard.equal",
				Self::GraveAccent => "key.keyboard.grave.accent",
				Self::LeftBracket => "key.keyboard.left.bracket",
				Self::Minus => "key.keyboard.minus",
				Self::Period => "key.keyboard.period",
				Self::RightBracket => "key.keyboard.right.bracket",
				Self::Semicolon => "key.keyboard.semicolon",
				Self::Slash => "key.keyboard.slash",
				Self::Space => "key.keyboard.space",
				Self::Tab => "key.keyboard.tab",
				Self::LeftAlt => "key.keyboard.left.alt",
				Self::LeftControl => "key.keyboard.left.control",
				Self::LeftShift => "key.keyboard.left.shift",
				Self::LeftSystem => "key.keyboard.left.win",
				Self::RightAlt => "key.keyboard.right.alt",
				Self::RightControl => "key.keyboard.right.control",
				Self::RightShift => "key.keyboard.right.shift",
				Self::RightSystem => "key.keyboard.right.win",
				Self::Enter => "key.keyboard.enter",
				Self::Escape => "key.keyboard.escape",
				Self::Backspace => "key.keyboard.backspace",
				Self::Delete => "key.keyboard.delete",
				Self::End => "key.keyboard.end",
				Self::Home => "key.keyboard.home",
				Self::Insert => "key.keyboard.insert",
				Self::PageDown => "key.keyboard.page.down",
				Self::PageUp => "key.keyboard.page.up",
				Self::CapsLock => "key.keyboard.caps.lock",
				Self::Pause => "key.keyboard.pause",
				Self::ScrollLock => "key.keyboard.scroll.lock",
				Self::Menu => "key.keyboard.menu",
				Self::PrintScreen => "key.keyboard.print.screen",
				Self::World1 => "key.keyboard.world.1",
				Self::World2 => "key.keyboard.world.2",
			}
		)
	}
}

impl ToInt for Keybind {
	/// Convert to the keycode used before 1.13
	fn to_int(&self) -> i32 {
		match self {
			Self::Unbound => 0,
			Self::Escape => 1,
			Self::Num1 => 2,
			Self::Num2 => 3,
			Self::Num3 => 4,
			Self::Num4 => 5,
			Self::Num5 => 6,
			Self::Num6 => 7,
			Self::Num7 => 8,
			Self::Num8 => 9,
			Self::Num9 => 10,
			Self::Num0 => 11,
			Self::Minus => 12,
			Self::Equal => 13,
			Self::Backspace => 14,
			Self::Tab => 15,
			Self::Q => 16,
			Self::W => 17,
			Self::E => 18,
			Self::R => 19,
			Self::T => 20,
			Self::Y => 21,
			Self::U => 22,
			Self::I => 23,
			Self::O => 24,
			Self::P => 25,
			Self::LeftBracket => 26,
			Self::RightBracket => 27,
			Self::Enter => 28,
			Self::LeftControl => 29,
			Self::A => 30,
			Self::S => 31,
			Self::D => 32,
			Self::F => 33,
			Self::G => 34,
			Self::H => 35,
			Self::J => 36,
			Self::K => 37,
			Self::L => 38,
			Self::Semicolon => 39,
			Self::Apostrophe => 40,
			Self::GraveAccent => 41,
			Self::LeftShift => 42,
			Self::Backslash => 43,
			Self::Z => 44,
			Self::X => 45,
			Self::C => 46,
			Self::V => 47,
			Self::B => 48,
			Self::N => 49,
			Self::M => 50,
			Self::Comma => 51,
			Self::Period => 52,
			Self::Slash => 53,
			Self::RightShift => 54,
			Self::NumpadMultiply => 55,
			Self::LeftAlt => 56,
			Self::Space => 57,
			Self::CapsLock => 58,
			Self::F1 => 59,
			Self::F2 => 60,
			Self::F3 => 61,
			Self::F4 => 62,
			Self::F5 => 63,
			Self::F6 => 64,
			Self::F7 => 65,
			Self::F8 => 66,
			Self::F9 => 67,
			Self::F10 => 68,
			Self::NumLock => 69,
			Self::ScrollLock => 70,
			Self::Numpad7 => 71,
			Self::Numpad8 => 72,
			Self::Numpad9 => 73,
			Self::NumpadSubtract => 74,
			Self::Numpad4 => 75,
			Self::Numpad5 => 76,
			Self::Numpad6 => 77,
			Self::NumpadAdd => 78,
			Self::Numpad1 => 79,
			Self::Numpad2 => 80,
			Self::Numpad3 => 81,
			Self::Numpad0 => 82,
			Self::NumpadDecimal => 83,
			Self::F11 => 87,
			Self::F12 => 88,
			Self::F13 => 100,
			Self::F14 => 101,
			Self::F15 => 102,
			Self::NumpadEqual => 141,
			Self::NumpadEnter => 156,
			Self::RightControl => 157,
			Self::NumpadDivide => 181,
			Self::RightAlt => 184,
			Self::Pause => 197,
			Self::Home => 199,
			Self::Up => 200,
			Self::PageUp => 201,
			Self::Left => 203,
			Self::Right => 205,
			Self::End => 207,
			Self::Down => 208,
			Self::PageDown => 209,
			Self::Insert => 210,
			Self::Delete => 211,
			Self::LeftSystem => 219,
			Self::RightSystem => 220,
			Self::MouseLeft => -100,
			Self::MouseRight => -99,
			Self::MouseMiddle => -98,
			Self::Mouse4 => -97,
			Self::Mouse5 => -96,
			Self::Mouse6 => -95,
			Self::Mouse7 => -94,
			Self::Mouse8 => -93,
			_ => 0,
		}
	}
}

impl Keybind {
	/// Returns either the key string or key code based on the minecraft version
	pub fn get_keycode(&self, before_1_13: bool) -> String {
		if before_1_13 {
			self.to_int().to_string()
		} else {
			self.to_string()
		}
	}
}
