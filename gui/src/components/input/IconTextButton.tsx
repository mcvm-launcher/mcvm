import { JSXElement } from "solid-js";
import Icon, { HasWidthHeight } from "../Icon";
import "./IconTextButton.css";

export default function IconTextButton(props: IconTextButtonProps) {
	const colorStyle = props.selected
		? `background-color:${props.color};border-color:${props.selectedColor}`
		: `background-color:${props.color};border-color:${props.color}`;

	return (
		<button
			class="icon-text-button bold"
			style={`${colorStyle}`}
			onClick={props.onClick}
		>
			<div class="icon-text-button-icon center">
				<Icon icon={props.icon} size={`calc(${props.size} * 0.7)`} />
			</div>
			<div class="icon-text-button-text">{props.text}</div>
		</button>
	);
}

export interface IconTextButtonProps {
	icon: (props: HasWidthHeight) => JSXElement;
	text: string;
	color: string;
	selectedColor: string;
	size: string;
	selected: boolean;
	onClick: () => void;
}
