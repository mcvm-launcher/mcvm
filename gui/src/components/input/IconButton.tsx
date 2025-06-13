import { JSXElement } from "solid-js";
import "./IconButton.css";
import Icon, { HasWidthHeight } from "../Icon";

export default function IconButton(props: IconButtonProps) {
	let backgroundColor = () =>
		props.selected ? props.selectedColor : props.color;

	let border = () =>
		props.border == undefined
			? `border-color: ${backgroundColor()}`
			: `border-color: ${props.border}`;

	let colorStyle = () => `background-color:${backgroundColor()};${border()}`;

	let iconColorStyle =
		props.iconColor == undefined ? "" : `color:${props.iconColor}`;

	return (
		<div
			class="cont icon-button"
			style={`${colorStyle()};width:${props.size};height:${
				props.size
			};${iconColorStyle}`}
			onClick={props.onClick}
		>
			<Icon icon={props.icon} size={`calc(${props.size} * 0.7)`} />
		</div>
	);
}

export interface IconButtonProps {
	icon: (props: HasWidthHeight) => JSXElement;
	color: string;
	selectedColor: string;
	iconColor?: string;
	border?: string;
	size: string;
	selected: boolean;
	onClick: (e: Event) => void;
}
