import { createSignal, JSXElement, Show } from "solid-js";
import Icon, { HasWidthHeight } from "../Icon";
import "./IconTextButton.css";

export default function IconTextButton(props: IconTextButtonProps) {
	let [isHovered, setIsHovered] = createSignal(false);

	let selectedBg =
		props.selectedBg == undefined ? props.color : props.selectedBg;

	const colorStyle = () =>
		props.selected
			? `background-color:${selectedBg};border-color:${props.selectedColor}`
			: isHovered()
			? `background-color:${props.color};border-color:var(--bg4)`
			: `background-color:${props.color};border-color:var(--bg3)`;

	let shadow = props.shadow == undefined ? true : props.shadow;

	return (
		<button
			class={`${shadow ? "input-shadow" : ""} icon-text-button bold`}
			style={`${colorStyle()}`}
			onClick={props.onClick}
			onmouseenter={() => setIsHovered(true)}
			onmouseleave={() => setIsHovered(false)}
		>
			<Show when={props.icon != undefined}>
				<div class="icon-text-button-icon center">
					<Icon icon={props.icon!} size={`calc(${props.size} * 0.7)`} />
				</div>
			</Show>
			<div class="icon-text-button-text">{props.text}</div>
		</button>
	);
}

export interface IconTextButtonProps {
	icon?: (props: HasWidthHeight) => JSXElement;
	text: string;
	color: string;
	selectedColor: string;
	selectedBg?: string;
	shadow?: boolean;
	size: string;
	selected: boolean;
	onClick: () => void;
}
