use std::rc::Rc;

use freya::query::UseMutation;
use nitrolaunch::plugin_crate::hook::hooks::DropdownButton;

use crate::{
	components::{dialog::tip::Tipped, input::Derivable},
	ops::{
		ToastedMutation,
		plugin_results::{
			OpenCustomPopup, OpenCustomPopupKeys, RunCustomAction, RunCustomActionKeys,
		},
	},
	prelude::*,
	theme::Colorway,
};

#[derive(PartialEq)]
pub struct InlineSelect<T: PartialEq + Clone> {
	options: Vec<SelectOption<T>>,
	selected: Selected<T>,
	on_select: NotEq<Rc<dyn Fn(Selected<T>)>>,
	derived_option: Option<T>,
	align_end: bool,
	fit: bool,
	grid_cols: u8,
}

#[allow(dead_code)]
impl<T: PartialEq + Clone> InlineSelect<T> {
	pub fn new(selected: Selected<T>, on_select: Rc<dyn Fn(Selected<T>)>) -> Self {
		Self {
			options: Vec::new(),
			selected,
			on_select: NotEq(on_select),
			derived_option: None,
			align_end: false,
			fit: false,
			grid_cols: 0,
		}
	}

	pub fn child(mut self, child: SelectOption<T>) -> Self {
		self.options.push(child);
		self
	}

	pub fn maybe_child(mut self, show: bool, child: impl FnOnce() -> SelectOption<T>) -> Self {
		if show {
			self.options.push(child());
		}
		self
	}

	pub fn children(mut self, children: impl IntoIterator<Item = SelectOption<T>>) -> Self {
		for child in children {
			self = self.child(child);
		}
		self
	}

	pub fn align_end(mut self) -> Self {
		self.align_end = true;
		self
	}

	/// Make the options fit the content instead of expanding to the full width
	pub fn fit(mut self) -> Self {
		self.fit = true;
		self
	}

	/// Make the options display in a grid with the given number of columns
	pub fn grid(mut self, cols: u8) -> Self {
		self.grid_cols = cols;
		self
	}
}

impl<T: PartialEq + Clone> InlineSelect<Option<T>> {
	#[allow(dead_code)]
	pub fn allow_none(mut self) -> Self {
		self.options.push(SelectOption::none());
		self
	}

	pub fn allow_inherit(mut self) -> Self {
		self.options
			.push(SelectOption::new(None, "Inherit", Some("diagram")));
		self
	}
}

impl<T: PartialEq + Clone> Derivable<T> for InlineSelect<T> {
	fn derived(mut self, value: Option<T>) -> Self {
		self.derived_option = value;
		self
	}
}

impl<T: PartialEq + Clone + 'static> Component for InlineSelect<T> {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();

		let selected = self.selected.clone();
		let upper_on_select = self.on_select.clone();
		let on_select: NotEq<Rc<dyn Fn(T)>> = NotEq(Rc::new(move |option| {
			(upper_on_select.0)(selected.clone().select(&option));
		}));

		let selected = self.selected.clone();
		let upper_on_select = self.on_select.clone();
		let on_deselect: NotEq<Rc<dyn Fn(T)>> = NotEq(Rc::new(move |option| {
			(upper_on_select.0)(selected.clone().deselect(&option));
		}));

		let options = self.options.iter().map(|x| {
			let is_selected = self.selected.is_selected(&x.id);
			let is_derived =
				!is_selected && self.derived_option.as_ref().is_some_and(|y| y == &x.id);

			InlineSelectOption {
				option: x.clone(),
				on_select: on_select.clone(),
				on_deselect: on_deselect.clone(),
				is_selected,
				is_derived,
				fit: self.fit,
			}
			.into_element()
		});

		let out = rect()
			.width(Size::fill())
			.maybe(self.fit, |this| this.width(Size::auto()))
			.cont()
			.main_align(if self.align_end {
				Alignment::End
			} else {
				Alignment::Start
			});

		if self.grid_cols > 0 {
			out.child(grid(self.grid_cols, options).gap(theme.gap))
		} else {
			out.children(options)
		}
	}
}

#[derive(PartialEq)]
struct InlineSelectOption<T: PartialEq + Clone> {
	option: SelectOption<T>,
	on_select: NotEq<Rc<dyn Fn(T)>>,
	on_deselect: NotEq<Rc<dyn Fn(T)>>,
	is_selected: bool,
	is_derived: bool,
	fit: bool,
}

impl<T: PartialEq + Clone + 'static> Component for InlineSelectOption<T> {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let is_hovered = use_state(|| false);
		let front_state = use_front_state();

		let id = self.option.id.clone();
		let on_select = self.on_select.clone();
		let on_deselect = self.on_deselect.clone();
		let is_selected = self.is_selected;

		rect()
			.cont()
			.center()
			.corner_radius(theme.round)
			.height(Size::px(theme.input_height))
			.padding((6.0, 12.0))
			.maybe(!self.is_derived, |this| {
				this.item_colorway(&theme, *is_hovered.read(), self.is_selected)
			})
			.maybe(self.is_derived, |this| {
				this.border(theme.border(theme.template))
					.color(theme.template)
					.background(theme.template_bg)
			})
			.maybe(
				self.is_selected && self.option.selected_colorway.is_some(),
				|mut this| {
					this.get_style().borders.clear();
					this.colorway(self.option.selected_colorway.unwrap(), &theme)
				},
			)
			.maybe(self.is_selected, |this| this.font_weight(FontWeight::BOLD))
			.maybe(!self.fit, |this| this.width(Size::flex(1.0)))
			.on_press(move |_| {
				if is_selected {
					on_deselect.0(id.clone());
				} else {
					on_select.0(id.clone());
				}
			})
			.clickable()
			.maybe_child(self.option.icon.clone())
			.maybe(self.option.tip.is_some(), |this| {
				this.tip(&front_state, self.option.tip.as_deref().unwrap())
			})
			.maybe_child(
				Some(self.option.title.as_str())
					.filter(|x| !x.is_empty())
					.map(|x| label().text(x.to_string()).max_lines(1)),
			)
	}
}

#[derive(PartialEq)]
pub struct Dropdown<T: PartialEq + Clone> {
	selected: Selected<T>,
	on_select: NotEq<Rc<dyn Fn(Selected<T>)>>,
	options: Vec<SelectOption<T>>,
	derived_option: Option<T>,
	on_open_change: Option<EventHandler<bool>>,
	is_loading: bool,
	custom_header: Option<SelectOption<T>>,
	options_width: Option<f32>,
	align_options_right: bool,
	align_options_top: bool,
	options_position: Option<Position>,
	header_width: Size,
	panel_colorway: bool,
	hide_arrow: bool,
}

#[allow(dead_code)]
impl<T: PartialEq + Clone> Dropdown<T> {
	pub fn new(selected: Selected<T>, on_select: Rc<dyn Fn(Selected<T>)>) -> Self {
		Self {
			selected,
			on_select: NotEq(on_select),
			options: Vec::new(),
			derived_option: None,
			on_open_change: None,
			is_loading: false,
			custom_header: None,
			options_width: None,
			align_options_right: false,
			align_options_top: false,
			options_position: None,
			header_width: Size::fill(),
			panel_colorway: false,
			hide_arrow: false,
		}
	}

	pub fn from_state(state: State<T>) -> Self
	where
		T: 'static,
	{
		Self::new(
			Selected::Single(state.read().clone()),
			Rc::new(move |selected| {
				if let Selected::Single(value) = selected {
					state.clone().set(value);
				}
			}),
		)
	}

	pub fn child(mut self, child: SelectOption<T>) -> Self {
		self.options.push(child);
		self
	}

	pub fn maybe_child(mut self, show: bool, child: impl FnOnce() -> SelectOption<T>) -> Self {
		if show {
			self.options.push(child());
		}
		self
	}

	pub fn children(mut self, children: impl IntoIterator<Item = SelectOption<T>>) -> Self {
		for child in children {
			self = self.child(child);
		}
		self
	}

	pub fn custom_buttons(
		mut self,
		buttons: impl IntoIterator<Item = DropdownButton>,
		to_id: impl Fn(usize) -> T,
	) -> Self {
		for (i, button) in buttons.into_iter().enumerate() {
			self = self.child(SelectOption::from_dropdown_button(to_id(i), button));
		}
		self
	}

	pub fn on_open_change(mut self, handler: impl Into<EventHandler<bool>>) -> Self {
		self.on_open_change = Some(handler.into());
		self
	}

	pub fn loading(mut self, is_loading: bool) -> Self {
		self.is_loading = is_loading;
		self
	}

	pub fn custom_header(mut self, header: SelectOption<T>) -> Self {
		self.custom_header = Some(header);
		self
	}

	pub fn options_width(mut self, width: f32) -> Self {
		self.options_width = Some(width);
		self
	}

	pub fn align_options_right(mut self) -> Self {
		self.align_options_right = true;
		self
	}

	pub fn align_options_top(mut self) -> Self {
		self.align_options_top = true;
		self
	}

	pub fn options_position(mut self, position: Position) -> Self {
		self.options_position = Some(position);
		self
	}

	pub fn header_width(mut self, width: Size) -> Self {
		self.header_width = width;
		self
	}

	pub fn panel_colorway(mut self) -> Self {
		self.panel_colorway = true;
		self
	}

	pub fn hide_arrow(mut self) -> Self {
		self.hide_arrow = true;
		self
	}
}

impl<T: PartialEq + Clone> Dropdown<Option<T>> {
	pub fn allow_none(mut self) -> Self {
		self.options.push(SelectOption::none());
		self
	}

	pub fn allow_inherit(mut self) -> Self {
		self.options
			.push(SelectOption::new(None, "Inherit", Some("diagram")));
		self
	}
}

impl<T: PartialEq + Clone> Derivable<T> for Dropdown<T> {
	fn derived(mut self, value: Option<T>) -> Self {
		if let Selected::Single(selected) = &self.selected
			&& Some(selected) != value.as_ref()
		{
			self.derived_option = value;
		}
		self
	}
}

impl<T: PartialEq + Clone + 'static> Component for Dropdown<T> {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let is_hovered = use_state(|| false);
		let mut is_open = use_state(|| false);

		let is_open2 = is_open;
		let on_open_change = self.on_open_change.clone();
		use_side_effect(move || {
			if let Some(handler) = &on_open_change {
				handler.call(*is_open2.read());
			}
		});

		let selected = self.selected.clone();
		let upper_on_select = self.on_select.clone();
		let on_select: NotEq<Rc<dyn Fn(T)>> = NotEq(Rc::new(move |option| {
			(upper_on_select.0)(selected.clone().select(&option));
		}));

		let selected = self.selected.clone();
		let upper_on_select = self.on_select.clone();
		let on_deselect: NotEq<Rc<dyn Fn(T)>> = NotEq(Rc::new(move |option| {
			(upper_on_select.0)(selected.clone().deselect(&option));
		}));

		let fit_header = self.header_width == Size::Inner;

		let preview = if let Some(custom_header) = &self.custom_header {
			dropdown_option_contents(custom_header, fit_header, true, &theme)
		} else {
			match &self.selected {
				Selected::Single(selected) => {
					let (actual, color) = if let Some(derived_option) = &self.derived_option {
						(derived_option, theme.template)
					} else {
						(selected, theme.fg)
					};
					if let Some(option) = self.options.iter().find(|x| x.id == *actual) {
						dropdown_option_contents(option, fit_header, true, &theme).color(color)
					} else {
						dropdown_option_contents(
							&SelectOption::simple("Unknown option"),
							fit_header,
							true,
							&theme,
						)
					}
				}
				Selected::Multi(selected) => dropdown_option_contents(
					&SelectOption::simple(format!("{} selected", selected.len())),
					fit_header,
					true,
					&theme,
				),
			}
		};

		let arrow = if *is_open.read() {
			"angle_down"
		} else {
			"angle_right"
		};
		let arrow = icon(arrow, 16.0);
		let arrow = rect()
			.width(Size::px(theme.input_height))
			.height(Size::px(theme.input_height))
			.center()
			.child(arrow);
		let preview = preview
			.maybe(!self.hide_arrow, |this| this.child(arrow))
			.maybe(
				self.hide_arrow
					&& self
						.custom_header
						.as_ref()
						.is_some_and(|x| !x.title.is_empty()),
				|this| {
					this.padding(Gaps::new(
						0.0,
						(theme.input_height - 16.0) * 2.0 / 3.0,
						0.0,
						0.0,
					))
				},
			);

		let header = rect()
			.width(self.header_width.clone())
			.height(Size::px(theme.input_height))
			.corner_radius(theme.round)
			.simple_colorway(&theme, *is_hovered.read(), false)
			.maybe(self.panel_colorway, |this| {
				this.panel_colorway(&theme, *is_hovered.read(), false)
			})
			.hover(is_hovered)
			.on_press(move |ev: Event<PressEventData>| {
				ev.stop_propagation();
				is_open.toggle()
			})
			.center()
			.child(preview);

		let option_count = (self.options.len() as f32).min(7.5);
		// Don't include the gap after the last option
		let options_height =
			Size::px(theme.input_height * option_count + theme.gap * (option_count - 1.0));

		let options = if self.is_loading {
			rect()
				.expanded()
				.center()
				.child(CircularLoader::new().size(16.0))
				.into_element()
		} else {
			let selected = self.selected.clone();
			let derived_option = self.derived_option.clone();
			let options = self.options.clone();
			let len = self.options.len();
			let theme2 = theme.clone();

			VirtualScrollView::new(move |item, _| {
				let option = options.get(item.index).unwrap();

				let is_selected = selected.is_selected(&option.id);
				let is_derived = derived_option.as_ref().is_some_and(|y| y == &option.id);

				DropdownOption {
					option: option.clone(),
					on_select: on_select.clone(),
					on_deselect: on_deselect.clone(),
					is_selected,
					is_derived,
				}
				.into_element()
			})
			.length(self.options.len())
			.item_size(move |i| {
				if i == len - 1 {
					theme2.input_height
				} else {
					theme2.input_height + theme2.gap
				}
			})
			.width(Size::fill())
			.height(options_height)
			.into_element()
		};

		let options_width = self.options_width.map(Size::px).unwrap_or_else(Size::fill);

		let options_position = if let Some(position) = &self.options_position {
			position.clone()
		} else {
			let options_position = if self.align_options_right {
				Position::new_absolute().right(0.0)
			} else {
				Position::new_absolute().left(0.0)
			};
			let offset = theme.input_height + 8.0;
			if self.align_options_top {
				options_position.bottom(offset)
			} else {
				options_position.top(offset)
			}
		};

		let options = rect()
			.width(options_width)
			.maybe(fit_header && self.options_width.is_none(), |this| {
				this.min_width(Size::px(150.0))
			})
			.position(options_position)
			.layer(Layer::Overlay)
			.padding(theme.gap)
			.corner_radius(theme.round)
			.panel_colorway(&theme, false, false)
			.overlay_shadow()
			.on_pointer_leave(move |_| {
				is_open.set(false);
			})
			.child(options);

		header.maybe(*is_open.read(), |this| this.child(options))
	}
}

#[derive(PartialEq)]
struct DropdownOption<T: PartialEq + Clone> {
	option: SelectOption<T>,
	on_select: NotEq<Rc<dyn Fn(T)>>,
	on_deselect: NotEq<Rc<dyn Fn(T)>>,
	is_selected: bool,
	is_derived: bool,
}

impl<T: PartialEq + Clone + 'static> Component for DropdownOption<T> {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let is_hovered = use_state(|| false);

		let on_select = self.on_select.clone();
		let on_deselect = self.on_deselect.clone();
		let id = self.option.id.clone();
		let is_selected = self.is_selected;

		let (fg, bg, border) = if self.is_derived {
			(theme.template, theme.template_bg, theme.template)
		} else if self.is_selected {
			(
				theme.item_select_border,
				theme.item_select,
				theme.item_select_border,
			)
		} else if *is_hovered.read() {
			(theme.fg, theme.highlight, theme.panel)
		} else {
			(theme.fg, theme.panel, theme.panel)
		};

		let out = rect()
			.width(Size::fill())
			.height(Size::px(theme.input_height))
			.color(fg)
			.background(bg)
			.border(theme.border(border))
			.corner_radius(theme.round)
			.margin(Gaps::new(0.0, 0.0, theme.gap, 0.0))
			.maybe(self.is_selected, |this| this.font_weight(FontWeight::BOLD))
			.cont()
			.clickable()
			.hover(is_hovered)
			.on_press(move |_| {
				if is_selected {
					(on_deselect.0)(id.clone());
				} else {
					(on_select.0)(id.clone());
				}
			})
			.child(dropdown_option_contents(&self.option, false, false, &theme));

		if let Some(tip) = &self.option.tip {
			Tipped::new(out, tip).into_element()
		} else {
			out.into_element()
		}
	}
}

fn dropdown_option_contents<T: PartialEq + Clone>(
	option: &SelectOption<T>,
	fit: bool,
	is_header: bool,
	theme: &Theme,
) -> Rect {
	rect()
		.horizontal()
		.flex()
		.maybe_child(option.icon.clone().map(|x| {
			rect()
				.width(Size::px(theme.input_height))
				.height(Size::px(theme.input_height))
				.center()
				.child(x)
		}))
		.maybe(!option.title.is_empty(), |this| {
			this.child(
				rect()
					.maybe(!fit, |this| this.width(Size::flex(1.0)))
					.height(Size::fill())
					.main_align(Alignment::Center)
					.cross_align(Alignment::Center)
					.maybe(option.icon.is_some(), |this| {
						this.cross_align(Alignment::Start)
					})
					.child(option.title.as_str()),
			)
		})
		.maybe(!is_header && option.action_button.is_some(), |this| {
			this.child(
				rect()
					.width(Size::px(theme.input_height))
					.height(Size::px(theme.input_height))
					.center()
					.child(option.action_button.clone().unwrap()),
			)
		})
}

#[derive(PartialEq, Clone)]
pub struct SelectOption<T: PartialEq + Clone> {
	pub id: T,
	pub title: String,
	pub icon: Option<Element>,
	pub tip: Option<String>,
	pub selected_colorway: Option<Colorway>,
	pub action_button: Option<Element>,
}

impl<T: PartialEq + Clone> SelectOption<T> {
	pub fn simple(id: T) -> Self
	where
		T: ToString,
	{
		let title = id.to_string();
		Self::new(id, &title, None)
	}

	pub fn simple_icon(id: T, ico: &str) -> Self {
		Self::new(id, "", Some(ico))
	}

	pub fn new(id: impl Into<T>, title: &str, ico: Option<&str>) -> Self {
		Self {
			id: id.into(),
			title: title.into(),
			icon: ico.map(|x| icon(x, 16.0).into_element()),
			tip: None,
			selected_colorway: None,
			action_button: None,
		}
	}

	pub fn new_custom_icon(id: impl Into<T>, title: &str, ico: Element) -> Self {
		Self {
			id: id.into(),
			title: title.into(),
			icon: Some(ico),
			tip: None,
			selected_colorway: None,
			action_button: None,
		}
	}

	pub fn from_dropdown_button(id: T, button: DropdownButton) -> Self {
		Self {
			id,
			title: button.text,
			icon: Some(icon(&button.icon, 16.0).into_element()),
			tip: button.tip,
			selected_colorway: None,
			action_button: None,
		}
	}

	pub fn tip(mut self, tip: &str) -> Self {
		self.tip = Some(tip.into());
		self
	}

	pub fn selected_colorway(mut self, colorway: Colorway) -> Self {
		self.selected_colorway = Some(colorway);
		self
	}

	pub fn action_button(mut self, button: Element) -> Self {
		self.action_button = Some(button);
		self
	}
}

impl<T: Clone + PartialEq> SelectOption<Option<T>> {
	pub fn simple_or_none(id: Option<T>) -> Self
	where
		T: ToString,
	{
		if let Some(id) = id {
			let title = id.to_string();
			Self::new(id, &title, None)
		} else {
			Self::none()
		}
	}

	pub fn none() -> Self {
		Self::new(None, "None", None)
	}
}

/// What's actually selected for a select component, supporting both single and multi select
#[derive(PartialEq, Clone)]
pub enum Selected<T: PartialEq + Clone> {
	Single(T),
	Multi(Vec<T>),
}

impl<T: PartialEq + Clone> Selected<T> {
	/// Gets a single result out, panicking if it is none
	pub fn single(self) -> T {
		match self {
			Self::Single(value) => value,
			_ => unreachable!(),
		}
	}

	/// Gets a single result out
	pub fn single_optional(self) -> Option<T> {
		match self {
			Self::Single(value) => Some(value),
			Self::Multi(values) => values.first().cloned(),
		}
	}

	/// Gets multiple results out
	pub fn multi(self) -> Vec<T> {
		match self {
			Self::Single(value) => vec![value],
			Self::Multi(values) => values,
		}
	}

	/// Checks whether this option is selected
	fn is_selected(&self, option: &T) -> bool {
		match self {
			Self::Single(value) => value == option,
			Self::Multi(values) => values.iter().any(|x| x == option),
		}
	}

	fn select(self, new: &T) -> Self {
		match self {
			Self::Single(..) => Self::Single(new.clone()),
			Self::Multi(mut list) => {
				list.push(new.clone());
				Self::Multi(list)
			}
		}
	}

	fn deselect(self, value: &T) -> Self {
		match self {
			Self::Single(..) => self,
			Self::Multi(list) => {
				let list = list.into_iter().filter(|x| x != value).collect();
				Self::Multi(list)
			}
		}
	}
}

pub fn run_dropdown_button(
	button: &DropdownButton,
	related_id: Option<String>,
	custom_action: &UseMutation<ToastedMutation<RunCustomAction>>,
	open_popup: &UseMutation<OpenCustomPopup>,
) {
	if let Some(action) = &button.action {
		custom_action.mutate(RunCustomActionKeys {
			plugin: button.plugin.clone(),
			action: action.clone(),
			params: serde_json::Value::Null,
			related_id: related_id.clone(),
			control_state: serde_json::Map::new(),
		});
	}
	if let Some(popup) = &button.popup {
		open_popup.mutate(OpenCustomPopupKeys {
			plugin: button.plugin.clone(),
			popup_id: popup.clone(),
			related_id,
		});
	}
}
