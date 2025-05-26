import { JSXElement } from "solid-js";
import "./IconButton.css";
import Icon, { HasWidthHeight } from "../Icon";

export default function IconButton(props: IconButtonProps) {
	const colorStyle = props.selected
		? `background-color:${props.selectedColor};border-color:${props.selectedColor}`
		: `background-color:${props.color};border-color:${props.color}`;

	let iconColorStyle =
		props.iconColor == undefined ? "" : `color:${props.iconColor}`;

	return (
		<div
			class="cont icon-button border"
			style={`${colorStyle};width:${props.size};height:${props.size};${iconColorStyle}`}
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
	size: string;
	selected: boolean;
	onClick: (e: Event) => void;
}
