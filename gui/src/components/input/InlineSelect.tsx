import { For, JSX, Show } from "solid-js";
import "./InlineSelect.css";

export default function InlineSelect(props: InlineSelectProps) {
	let columns = props.columns == undefined ? 3 : props.columns;
	let connected = props.connected == undefined ? true : props.connected;
	let grid = props.grid == undefined ? true : props.grid;
	let solidSelect = props.solidSelect == undefined ? false : props.solidSelect;

	return (
		<div
			class={`${connected ? "input-shadow" : ""} inline-select ${
				connected ? "connected" : "disconnected"
			}`}
			style={`display:${
				grid ? "grid" : "flex"
			};grid-template-columns:repeat(${columns}, minmax(0, 1fr))`}
		>
			<Show when={props.allowEmpty == undefined ? false : props.allowEmpty}>
				<InlineSelectOption
					option={{
						value: undefined,
						contents: "None",
					}}
					connected={connected}
					onSelect={props.onChange}
					selected={props.selected}
					isLast={props.selected == props.options[0].value}
					isFirst={true}
					class={props.optionClass}
					solidSelect={solidSelect}
				/>
			</Show>
			<For each={props.options}>
				{(option, index) => (
					<InlineSelectOption
						option={option}
						connected={connected}
						onSelect={props.onChange}
						selected={props.selected}
						isLast={index() == props.options.length - 1}
						isFirst={index() == 0 && !props.allowEmpty}
						class={props.optionClass}
						solidSelect={solidSelect}
					/>
				)}
			</For>
		</div>
	);
}

export interface InlineSelectProps {
	options: Option[];
	selected?: string;
	onChange: (option: string | undefined) => void;
	columns?: number;
	allowEmpty?: boolean;
	connected?: boolean;
	optionClass?: string;
	grid?: boolean;
	solidSelect?: boolean;
}

function InlineSelectOption(props: OptionProps) {
	let isSelected = () => props.selected == props.option.value;
	let color =
		props.option.color == undefined ? "var(--fg2)" : props.option.color;

	let textColor = () =>
		props.solidSelect && isSelected() ? "black" : "var(--fg)";
	let backgroundColor = () =>
		props.solidSelect && isSelected()
			? color
			: props.connected
			? "var(--bg0)"
			: "var(--bg2)";
	let borderColor = () => `border-color:${isSelected() ? color : "var(--bg3)"}`;

	return (
		<div
			class={`cont inline-select-option ${
				props.connected ? "connected" : "disconnected input-shadow"
			} ${props.class == undefined ? "" : props.class} ${
				isSelected() ? "selected" : ""
			} ${props.isLast ? "last" : "not-last"} ${
				props.isFirst ? "" : "not-first"
			}`}
			style={`${borderColor()};color:${textColor()};background-color:${backgroundColor()}`}
			onclick={() => props.onSelect(props.option.value)}
		>
			{props.option.contents}
		</div>
	);
}

interface OptionProps {
	option: Option;
	selected?: string;
	connected: boolean;
	solidSelect: boolean;
	class?: string;
	isFirst: boolean;
	isLast: boolean;
	onSelect: (option: string | undefined) => void;
}

export interface Option {
	value: string | undefined;
	contents: JSX.Element;
	color?: string;
}
