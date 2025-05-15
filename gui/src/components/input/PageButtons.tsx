import { Show } from "solid-js";
import "./PageButtons.css";

export default function PageButtons(props: PageButtonsProps) {
	return (
		<div class="cont page-buttons">
			<Show when={props.page > 1}>
				<PageButton
					page={props.page - 1}
					onclick={() => props.pageFunction(props.page - 2)}
				/>
			</Show>
			<Show when={props.page > 0}>
				<PageButton
					page={props.page}
					onclick={() => props.pageFunction(props.page - 1)}
				/>
			</Show>
			<PageButton
				page={props.page + 1}
				onclick={() => props.pageFunction(props.page)}
				selected
			/>
			<Show when={props.page < props.pageCount}>
				<PageButton
					page={props.page + 2}
					onclick={() => props.pageFunction(props.page + 1)}
				/>
			</Show>
			<Show when={props.page < props.pageCount - 1}>
				<PageButton
					page={props.page + 3}
					onclick={() => props.pageFunction(props.page + 2)}
				/>
			</Show>
		</div>
	);
}

function PageButton(props: PageButtonProps) {
	let selectedClass = props.selected ? " selected" : "";
	return (
		<button class={`cont page-button${selectedClass}`} onclick={props.onclick}>
			{props.page}
		</button>
	);
}

interface PageButtonProps {
	page: number;
	selected?: boolean;
	onclick: () => void;
}

export interface PageButtonsProps {
	page: number;
	pageCount: number;
	pageFunction: (page: number) => void;
}
