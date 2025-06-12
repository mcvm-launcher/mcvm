import { createSignal, Match, Switch } from "solid-js";
import { Delete, Search } from "../../icons";
import "./SearchBar.css";

export default function SearchBar(props: SearchBarProps) {
	let [term, setTerm] = createSignal<string | undefined>(props.value);

	return (
		<form
			class="cont input-shadow search-bar"
			onsubmit={(e) => {
				e.preventDefault();
				let term2 = term() == undefined ? "" : term()!;
				props.method(term2);
			}}
		>
			<div class="cont search-bar-icon">
				<Switch>
					<Match when={term() == undefined || term() == ""}>
						<Search />
					</Match>
					<Match when={term() != undefined && term() != ""}>
						<div
							class="cont"
							style="cursor:pointer"
							onclick={() => {
								setTerm("");
							}}
						>
							<Delete />
						</div>
					</Match>
				</Switch>
			</div>
			<input
				type="text"
				placeholder={props.placeholder}
				value={term() == undefined ? "" : term()}
				onkeyup={(e) => setTerm(e.target.value)}
			/>
		</form>
	);
}

export interface SearchBarProps {
	placeholder?: string;
	value?: string;
	method: (term: string) => void;
}
