import { JSX, Show } from "solid-js";
import PageBlock from "../PageBlock";
import "./Modal.css";

export default function Modal(props: ModalProps) {
	return (
		<Show when={props.visible}>
			<PageBlock onClick={() => props.onClose(false)} />
			<div class="cont modal-container">
				<div
					class="cont modal-behind"
					onclick={() => props.onClose(false)}
				></div>
				<div class="cont modal" style={`width:${props.width}`}>
					{props.children}
				</div>
			</div>
		</Show>
	);
}

export interface ModalProps {
	children: JSX.Element;
	visible: boolean;
	width: string;
	onClose: (visible: boolean) => void;
}
