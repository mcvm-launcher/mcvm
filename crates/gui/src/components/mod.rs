use std::time::Duration;

use freya::{
	components::{
		Button, ButtonColorsThemePartialExt, ButtonLayoutThemePartialExt, ImageViewer, Skeleton,
		SkeletonThemePartialExt,
	},
	elements::{
		extensions::{ContainerPositionExt, EventHandlersExt, LayerExt, StyleExt, TextStyleExt},
		label::{Label, label},
		rect::Rect,
	},
	prelude::{
		Border, BorderAlignment, BorderWidth, ChildrenExt, Color, Component, ContainerExt,
		ContainerSizeExt, ContainerWithContentExt, Content, Cursor, Element, IntoElement, Layer,
		Position, Size, State, TextOverflow, WritableUtils, rect,
	},
	winit::window::CursorIcon,
};
use freya_core::style::corner_radius::CornerRadius;
use reqwest::Url;

use crate::{
	icons::icon,
	theme::{Colorway, Theme},
};

pub mod account;
pub mod console;
pub mod dialog;
pub mod footer;
pub mod global;
pub mod input;
pub mod instance;
pub mod markdown;
pub mod misc;
pub mod nav;
pub mod output_indicator;
pub mod pkg;
pub mod tag;
pub mod gallery;

pub const TOAST_TIP_LAYER: u8 = 3;

pub fn segment(child: impl IntoElement, width: f32) -> Rect {
	rect().width(Size::flex(width)).child(child)
}

pub fn img(url: &str) -> ImageViewer {
	ImageViewer::new(Url::parse(url).unwrap_or(Url::parse("https://example.com").unwrap()))
		.error_renderer(|e| {
			eprintln!("Image load error: {e}");
			icon("error", 16.0).into_element()
		})
		.asset_age(Duration::from_mins(3))
}

pub fn button(theme: &Theme) -> Button {
	Button::new()
		.color(theme.fg)
		.background(Color::TRANSPARENT)
		.hover_background(theme.item_hover)
		.border_fill(Color::TRANSPARENT)
		.corner_radius(theme.round)
		.padding(theme.gap)
		.cursor_icon(CursorIcon::Pointer)
}

pub fn icon_button(icon: &str, theme: &Theme) -> Button {
	button(theme).child(crate::icons::icon(icon, 16.0))
}

pub fn icon_text_button(icon: &str, text: &str, theme: &Theme) -> Button {
	elem_text_button(crate::icons::icon(icon, 16.0), text, theme)
}

pub fn elem_text_button(elem: impl IntoElement, text: &str, theme: &Theme) -> Button {
	button(theme)
		.height(Size::px(theme.input_height))
		.padding(theme.gap2)
		.child(rect().cont().center().child(elem).child(text))
}

pub fn skeleton(width: Size, height: Size, theme: &Theme) -> Skeleton {
	Skeleton::new(true)
		.width(width)
		.height(height)
		.background(theme.item_hover)
		.corner_radius(theme.round)
}

pub fn placeholder(text: &str, theme: &Theme) -> Rect {
	rect().expanded().center().child(text).color(theme.fg3)
}

pub fn clip_text(text: &str) -> Label {
	label()
		.text(text.to_string())
		.width(Size::fill())
		.max_lines(1)
		.text_overflow(TextOverflow::Ellipsis)
}

pub trait CustomStyles {
	/// Sets full width and height
	fn fill(self) -> Self;

	/// Sets a gap and horizontal layout
	fn cont(self) -> Self;

	/// Sets flex content
	fn flex(self) -> Self;

	/// Sets a colorway on this element
	fn colorway(self, colorway: Colorway, theme: &Theme) -> Self;

	/// Sets full item colorway based off hover / select state
	fn item_colorway(self, theme: &Theme, hovered: bool, selected: bool) -> Self;

	/// Sets full panel colorway based off hover / select state
	fn panel_colorway(self, theme: &Theme, hovered: bool, selected: bool) -> Self;

	/// Sets full panel colorway based off hover / select state
	fn simple_colorway(self, theme: &Theme, hovered: bool, selected: bool) -> Self;

	/// Sets a derived colorway
	fn derived_colorway(self, theme: &Theme) -> Self;
}

impl<T: ContainerSizeExt + StyleExt + ContainerWithContentExt + TextStyleExt> CustomStyles for T {
	fn fill(self) -> Self {
		self.width(Size::fill()).height(Size::fill())
	}

	fn cont(self) -> Self {
		self.horizontal().spacing(6.0).flex()
	}

	fn flex(self) -> Self {
		self.content(Content::Flex)
	}

	fn colorway(self, colorway: Colorway, theme: &Theme) -> Self {
		self.color(colorway.fg)
			.border(theme.border(colorway.border))
			.background(colorway.bg)
	}

	fn item_colorway(self, theme: &Theme, hovered: bool, selected: bool) -> Self {
		let (fg, border, bg) = item_colorway(theme, hovered, selected);

		self.color(fg)
			.border(Some(Border {
				fill: border,
				width: theme.border.into(),
				alignment: BorderAlignment::Inner,
			}))
			.background(bg)
	}

	fn panel_colorway(self, theme: &Theme, hovered: bool, selected: bool) -> Self {
		let (fg, border, bg) = panel_colorway(theme, hovered, selected);

		self.color(fg)
			.border(Some(Border {
				fill: border,
				width: theme.border.into(),
				alignment: BorderAlignment::Inner,
			}))
			.background(bg)
	}

	fn simple_colorway(self, theme: &Theme, hovered: bool, selected: bool) -> Self {
		let bg = simple_colorway(theme, hovered, selected);

		self.background(bg)
	}

	fn derived_colorway(self, theme: &Theme) -> Self {
		self.color(theme.template)
			.border(theme.border(theme.template))
			.background(theme.template_bg)
	}
}

pub trait FancyBorderExt {
	/// Adds a shiny transparent border
	fn shiny_border(self, theme: &Theme) -> Self;
}

impl FancyBorderExt for Rect {
	fn shiny_border(mut self, theme: &Theme) -> Self {
		let radius = self.get_style().corner_radius;
		self.child(
			rect()
				.expanded()
				.position(Position::new_absolute())
				.border(theme.border(0x33cccccc))
				.layer(Layer::Relative(2))
				.corner_radius(radius),
		)
	}
}

pub trait FancyBorderExtImage {
	/// Adds a shiny transparent border
	fn shiny_border(self, corner_radius: impl Into<CornerRadius>, theme: &Theme) -> Self;
}

impl FancyBorderExtImage for ImageViewer {
	fn shiny_border(self, corner_radius: impl Into<CornerRadius>, theme: &Theme) -> Self {
		self.child(
			rect()
				.expanded()
				.position(Position::new_absolute())
				.border(theme.border(0x33cccccc))
				.layer(Layer::Relative(2))
				.corner_radius(corner_radius),
		)
	}
}

/// Picks background color from hover and select state for an item
pub fn simple_colorway(theme: &Theme, hovered: bool, selected: bool) -> Color {
	if selected {
		theme.item_select
	} else if hovered {
		theme.panel_hover
	} else {
		Color::TRANSPARENT
	}
}

/// Picks foreground, border and background colors from hover and select state for an item
pub fn item_colorway(theme: &Theme, hovered: bool, selected: bool) -> (Color, Color, Color) {
	if selected {
		(
			theme.item_select_border,
			theme.item_select_border,
			theme.item_select,
		)
	} else if hovered {
		(theme.fg, theme.item_border, theme.item_hover)
	} else {
		(theme.fg, theme.item_border, theme.item)
	}
}

/// Picks foreground, border and background colors from hover and select state for a panel
pub fn panel_colorway(theme: &Theme, hovered: bool, selected: bool) -> (Color, Color, Color) {
	if selected {
		(
			theme.item_select_border,
			theme.item_select_border,
			theme.item_select,
		)
	} else if hovered {
		(theme.fg, theme.panel_border, theme.panel_hover)
	} else {
		(theme.fg, theme.panel_border, theme.panel)
	}
}

pub trait CustomEvents {
	/// Sets cursor to pointer on mouse over
	fn clickable(self) -> Self;

	/// Updates a state with hover status
	fn hover(self, state: State<bool>) -> Self;

	// /// Extends an event handler
	// fn extend_event<T>(self, event: EventName, handler: EventHandler<T>) -> Self;
}

impl<T: EventHandlersExt> CustomEvents for T {
	fn clickable(self) -> Self {
		self.on_pointer_enter(|_| {
			Cursor::set(CursorIcon::Pointer);
		})
		.on_pointer_leave(|_| {
			Cursor::set(CursorIcon::default());
		})
	}

	fn hover(self, mut state: State<bool>) -> Self {
		self.on_pointer_over(move |_| {
			Cursor::set(CursorIcon::Pointer);
			state.set(true);
		})
		.on_pointer_out(move |_| {
			Cursor::set(CursorIcon::default());
			state.set(false);
		})
	}

	// fn extend_event(mut self, event: EventName, handler: EventHandlerType) -> Self {
	// 	fn extend_handler<T: Clone + 'static>(handler: EventHandler<T>, event: &mut EventHandler<T>) {
	// 		let old = event.clone();
	// 		*event = (move |arg: T| {
	// 			handler.call(arg.clone());
	// 			old.call(arg);
	// 		})
	// 		.into();
	// 	}

	// 	if let Some(event) = self.get_event_handlers().get_mut(&event) {
	// 		match (handler, event) {
	// 			(EventHandlerType::File(handler), EventHandlerType::File(event)) => {
	// 				extend_handler(handler, event)
	// 			}
	// 		}
	// 	} else {
	// 	}

	// 	self
	// }
}

pub trait ButtonExt {
	fn active(self, theme: &Theme) -> Self;
}

impl ButtonExt for Button {
	fn active(self, theme: &Theme) -> Self {
		self.color(theme.primary)
			.border_fill(theme.primary)
			.focus_border_fill(theme.primary)
			.background(theme.primary_bg)
			.hover_background(theme.primary_bg)
	}
}

pub fn grid<T: IntoElement + 'static>(cols: u8, items: impl IntoIterator<Item = T>) -> Grid {
	Grid {
		cols,
		gap: 0.0,
		items: items.into_iter().map(|x| x.into_element()).collect(),
	}
}

#[derive(PartialEq)]
pub struct Grid {
	cols: u8,
	gap: f32,
	items: Vec<Element>,
}

impl Grid {
	pub fn gap(mut self, gap: f32) -> Self {
		self.gap = gap;
		self
	}
}

impl Component for Grid {
	fn render(&self) -> impl IntoElement {
		let rows = self.items.chunks(self.cols as usize).map(|items| {
			rect()
				.horizontal()
				.width(Size::fill())
				// .spacing(self.gap)
				.children(items.iter().map(|x| {
					rect()
						.width(Size::percent(100.0 / (self.cols as f32)))
						.child(x.clone())
						.margin(self.gap / 2.0)
						.into_element()
				}))
				.into_element()
		});

		rect()
			.vertical()
			.width(Size::fill())
			.padding(self.gap / 2.0)
			.children(rows)
	}
}

pub fn border_bottom(width: f32, color: impl Into<Color>) -> Border {
	Border {
		fill: color.into(),
		width: BorderWidth {
			bottom: width,
			..Default::default()
		},
		alignment: BorderAlignment::Inner,
	}
}

pub fn border_top(width: f32, color: impl Into<Color>) -> Border {
	Border {
		fill: color.into(),
		width: BorderWidth {
			top: width,
			..Default::default()
		},
		alignment: BorderAlignment::Inner,
	}
}

pub fn border_right(width: f32, color: impl Into<Color>) -> Border {
	Border {
		fill: color.into(),
		width: BorderWidth {
			right: width,
			..Default::default()
		},
		alignment: BorderAlignment::Inner,
	}
}

pub fn border_left(width: f32, color: impl Into<Color>) -> Border {
	Border {
		fill: color.into(),
		width: BorderWidth {
			left: width,
			..Default::default()
		},
		alignment: BorderAlignment::Inner,
	}
}
