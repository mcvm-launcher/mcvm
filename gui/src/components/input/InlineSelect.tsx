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
					isLast={props.selected == props.options[0].value}
					isFirst={true}
				/>
			</Show>
			<For each={props.options}>
				{(option, index) => (
					<InlineSelectOption
						option={option}
						onSelect={props.onChange}
						selected={props.selected}
						isLast={
							index() == props.options.length - 1 ||
							props.selected == props.options[index() + 1].value
						}
						isFirst={index() == 0 && !props.allowEmpty}
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
			class={`cont inline-select-option ${isSelected() ? "selected" : ""} ${
				props.isLast ? "" : "not-last"
			} ${props.isFirst ? "" : "not-first"}`}
			style={`${isSelected() ? `border-color:${color}` : "inherit"}`}
			onclick={() => props.onSelect(props.option.value)}
		>
			{props.option.contents}
		</div>
	);
}

interface OptionProps {
	option: Option;
	selected?: string;
	isFirst: boolean;
	isLast: boolean;
	onSelect: (option: string | undefined) => void;
}

export interface Option {
	value: string | undefined;
	contents: JSX.Element;
	color?: string;
}
