import { createSignal, JSX, Show } from "solid-js";
import "./Tip.css";

export default function Tip(props: TipProps) {
	let [visible, setVisible] = createSignal(false);

	let side = props.side == undefined ? "right" : props.side;
	let fullwidth = props.fullwidth == undefined ? false : props.fullwidth;

	return (
		<div class="tip-container" style={`${fullwidth ? "width:100%" : ""}`}>
			<div
				onmouseenter={() => setVisible(true)}
				onmouseleave={() => setVisible(false)}
				style={`${fullwidth ? "width:100%" : ""}`}
			>
				{props.children}
			</div>
			<Show when={visible()}>
				<div class={`fade-in tip ${side}`}>
					<div class={`input-shadow tip-body ${side}`}>{props.tip}</div>
					<div class={`input-shadow tip-arrow ${side}`}></div>
				</div>
			</Show>
		</div>
	);
}

export interface TipProps {
	children: JSX.Element;
	tip: JSX.Element;
	side?: TipSide;
	fullwidth?: boolean;
}

export type TipSide = "top" | "bottom" | "right";
