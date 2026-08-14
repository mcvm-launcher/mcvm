use html_parser::{Dom, Node};
use nitrolaunch::shared::util::open_link;

use crate::prelude::*;

#[derive(PartialEq)]
pub struct MarkdownHTMLViewer {
	pub body: String,
}

impl Component for MarkdownHTMLViewer {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();

		MarkdownViewer::new(self.body.clone())
			.width(Size::fill())
			.paragraph_size(14.0)
			.padding(32.0)
			.color(theme.fg2)
			.code_font_size(14.0)
			.color_code(theme.fg2)
			.background_code(theme.item)
			.background_blockquote(theme.item)
			.background_divider(theme.item)
			.inline_element(move |html: String| {
				let Ok(dom) = Dom::parse(&html) else {
					return Some(html.to_string().into_element());
				};

				dom.children
					.into_iter()
					.next()
					.map(|x| node_to_elem(x, &theme))
			})
	}
}

#[derive(PartialEq)]
pub struct HTMLViewer {
	pub body: String,
}

impl Component for HTMLViewer {
	fn render(&self) -> impl IntoElement {
		let theme = use_theme();
		let body = self.body.clone();
		let dom = use_memo(move || Dom::parse(&body).unwrap_or_default());

		rect()
			.width(Size::fill())
			.spacing(theme.gap)
			.padding(32.0)
			.children(
				dom.read()
					.children
					.clone()
					.into_iter()
					.map(move |x| node_to_elem(x, &theme)),
			)
	}
}

fn node_to_elem(node: Node, theme: &Theme) -> Element {
	match node {
		Node::Comment(..) => rect().into_element(),
		Node::Text(text) => label().text(text.replace("&nbsp;", " ")).into_element(),
		Node::Element(el) => match el.name.as_str() {
			"p" => {
				let mut p = paragraph();
				for child in el.children.into_iter() {
					if let Node::Text(text) = &child {
						p = p.span(text.replace("&nbsp;", " "));
					} else {
						p = p.child(node_to_elem(child, theme));
					}
				}
				p.into_element()
			}
			"div" | "span" => rect()
				.spacing(theme.gap)
				.children(el.children.into_iter().map(|x| node_to_elem(x, theme)))
				.into_element(),
			"a" => {
				let text = get_single_text_child(&el);
				let href = el.attributes.get("href").cloned().unwrap_or_default();
				if let Some(text) = text {
					label()
						.text(text)
						.text_decoration(TextDecoration::Underline)
						.color(theme.primary)
						.clickable()
						.on_press(move |_| {
							if let Some(href) = &href {
								let _ = open_link(href);
							}
						})
						.into_element()
				} else {
					rect()
						.children(el.children.into_iter().map(|x| node_to_elem(x, theme)))
						.clickable()
						.on_press(move |_| {
							if let Some(href) = &href {
								let _ = open_link(href);
							}
						})
						.into_element()
				}
			}
			"h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
				let text = get_single_text_child(&el);
				let size = match el.name.as_str() {
					"h1" => 32.0,
					"h2" => 28.0,
					"h3" => 24.0,
					"h4" => 20.0,
					"h5" => 16.0,
					"h6" => 14.0,
					_ => 14.0,
				};

				label()
					.maybe(text.is_some(), |this| this.text(text.unwrap()))
					.font_size(size)
					.font_weight(FontWeight::BOLD)
					.into_element()
			}
			"em" => rect()
				.font_slant(FontSlant::Italic)
				.children(el.children.into_iter().map(|x| node_to_elem(x, theme)))
				.into_element(),
			"strong" => rect()
				.font_weight(FontWeight::BOLD)
				.children(el.children.into_iter().map(|x| node_to_elem(x, theme)))
				.into_element(),
			"br" => rect().height(Size::px(theme.gap2)).into_element(),
			"ul" => unordered_list(
				el.children.into_iter().map(|x| node_to_elem(x, theme)),
				theme,
			),
			"li" => el
				.children
				.into_iter()
				.next()
				.map(|x| node_to_elem(x, theme))
				.unwrap_or(rect().into_element()),
			"img" => {
				let src = el
					.attributes
					.get("src")
					.cloned()
					.flatten()
					.unwrap_or_default();
				let width = el
					.attributes
					.get("width")
					.cloned()
					.flatten()
					.and_then(|x| x.parse::<f32>().ok());
				let height = el
					.attributes
					.get("height")
					.cloned()
					.flatten()
					.and_then(|x| x.parse::<f32>().ok());

				if src.ends_with(".svg") {
					SvgViewer::new(
						Url::parse(&src).unwrap_or(Url::parse("https://example.com").unwrap()),
					)
					.maybe(width.is_some(), |this| this.width(Size::px(width.unwrap())))
					.maybe(height.is_some(), |this| {
						this.height(Size::px(height.unwrap()))
					})
					.into_element()
				} else {
					img(&src)
						.maybe(width.is_some(), |this| this.width(Size::px(width.unwrap())))
						.maybe(height.is_some(), |this| {
							this.height(Size::px(height.unwrap()))
						})
						.into_element()
				}
			}
			_ => rect().into_element(),
		},
	}
}

fn get_single_text_child(el: &html_parser::Element) -> Option<String> {
	if el.children.len() == 1 {
		if let Node::Text(text) = &el.children[0] {
			return Some(text.clone());
		}
	}
	None
}

pub fn unordered_list(items: impl IntoIterator<Item = impl IntoElement>, theme: &Theme) -> Element {
	rect()
		.width(Size::fill())
		.spacing(theme.gap)
		.children(items.into_iter().map(|item| {
			let bullet = rect()
				.width(Size::px(8.0))
				.height(Size::px(8.0))
				.corner_radius(4.0)
				.background(theme.fg);

			rect()
				.width(Size::fill())
				.horizontal()
				.cross_align(Alignment::Center)
				.spacing(theme.gap2)
				.child(bullet)
				.child(item.into_element())
				.into_element()
		}))
		.into_element()
}
