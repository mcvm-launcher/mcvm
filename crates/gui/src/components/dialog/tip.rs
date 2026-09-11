use std::rc::Rc;

use crate::{components::TOAST_TIP_LAYER, prelude::*, state::FrontState, util::Shared};

#[derive(PartialEq)]
pub struct Tips;

const OFFSET: f32 = 14.0;
const FLIP_MARGIN: f32 = 180.0;
const MAX_WIDTH: f32 = 250.0;

impl Component for Tips {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let front_state = use_front_state();
		front_state.read().subscribe(FrontChannel::Tip);
		let tip = front_state.read().tip().cloned().unwrap_or_default();
		let platform = Platform::get();
		let mut window = *platform.root_size.peek();
		let scale_factor = *platform.scale_factor.peek();
		window.width /= scale_factor as f32;
		window.height /= scale_factor as f32;
		let place_right = window.width - tip.x < FLIP_MARGIN;
		let place_bottom = window.height - tip.y < FLIP_MARGIN;

		let available_width = if place_right {
			(tip.x - OFFSET).clamp(0.0, MAX_WIDTH)
		} else {
			(window.width - tip.x - OFFSET).clamp(0.0, MAX_WIDTH)
		};
		let available_height = if place_bottom {
			(tip.y - OFFSET).max(0.0)
		} else {
			(window.height - tip.y - OFFSET).max(0.0)
		};

		let mut position = Position::new_global();
		position = if place_bottom {
			position.bottom((window.height - tip.y) + OFFSET)
		} else {
			position.top(tip.y + OFFSET)
		};
		position = if place_right {
			position.right((window.width - tip.x) + OFFSET)
		} else {
			position.left(tip.x + OFFSET)
		};

		// Nested layers instead of RelativeOverlay because it doesn't work
		rect()
			.position(position)
			.max_width(Size::px(available_width))
			.max_height(Size::px(available_height))
			.layer(Layer::OverlayLevel(TOAST_TIP_LAYER))
			.child(
				rect()
					.padding(theme.gap2)
					.item_colorway(&theme, false, false)
					.border(theme.border(theme.secondary))
					.corner_radius(theme.round)
					.overlay_shadow()
					.center()
					.cont()
					.maybe(front_state.read().tip().is_none(), |this| this.opacity(0.0))
					// Prevent quickly hovering the tip from freezing it in place
					.interactive(false)
					.child(icon("info", 12.0))
					.child(tip.tip.as_ref()),
			)
	}
}

#[derive(PartialEq, Clone, Default)]
pub struct Tip {
	pub x: f32,
	pub y: f32,
	pub tip: Rc<str>,
}

pub trait TipExt {
	fn tip(self, front_state: &Shared<FrontState>, tip: &str) -> Self;
}

impl<T: EventHandlersExt> TipExt for T {
	fn tip(self, front_state: &Shared<FrontState>, tip: &str) -> Self {
		let tip = tip.to_string();
		let front_state = front_state.clone();
		let front_state2 = front_state.clone();

		self.on_mouse_move(move |event: Event<MouseEventData>| {
			front_state.write().set_tip(Some(Tip {
				x: event.global_location.x as f32,
				y: event.global_location.y as f32,
				tip: tip.clone().into(),
			}));
		})
		.on_pointer_leave(move |_| {
			front_state2.write().set_tip(None);
		})
	}
}

#[derive(PartialEq)]
pub struct Tipped(Element, String);

impl Tipped {
	pub fn new(element: impl IntoElement, tip: &str) -> Self {
		Self(element.into_element(), tip.to_string())
	}
}

impl Component for Tipped {
	fn render(&self) -> impl IntoElement {
		let front_state = use_front_state();

		let front_state2 = front_state.clone();
		use_drop(move || front_state2.write().set_tip(None));

		rect().tip(&front_state, &self.1).child(self.0.clone())
	}
}
