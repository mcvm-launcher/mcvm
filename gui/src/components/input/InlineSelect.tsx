import { For, JSX, Show } from "solid-js";
import "./InlineSelect.css";

export default function InlineSelect(props: InlineSelectProps) {
	let columns = props.columns == undefined ? 3 : props.columns;

	return (
		<div
			class="inline-select"
			style={`grid-template-columns:repeat(${columns}, minmax(0, 1fr))`}
		>
			<Show when={props.allowEmpty == undefined ? false : props.allowEmpty}>
				<InlineSelectOption
					option={{
						value: undefined,
						contents: "None",
					}}
					onSelect={props.onChange}
					selected={props.selected}
				/>
			</Show>
			<For each={props.options}>
				{(option) => (
					<InlineSelectOption
						option={option}
						onSelect={props.onChange}
						selected={props.selected}
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
}

function InlineSelectOption(props: OptionProps) {
	console.log(props.selected);
	let isSelected = () => props.selected == props.option.value;
	let color =
		props.option.color == undefined ? "var(--fg2)" : props.option.color;

	return (
		<div
			class={`cont inline-select-option ${isSelected() ? "selected" : ""}`}
			style={`border-color:${color}`}
			onclick={() => props.onSelect(props.option.value)}
		>
			{props.option.contents}
		</div>
	);
}

interface OptionProps {
	option: Option;
	selected?: string;
	onSelect: (option: string | undefined) => void;
}

export interface Option {
	value: string | undefined;
	contents: JSX.Element;
	color?: string;
}
