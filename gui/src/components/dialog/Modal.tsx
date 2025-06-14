import { JSX, Show } from "solid-js";
import PageBlock from "../PageBlock";
import "./Modal.css";

export default function Modal(props: ModalProps) {
	return (
		<Show when={props.visible}>
			<PageBlock onClick={() => props.onClose(false)} />
			<div class="modal" style={`width:${props.width}`}>
				{props.children}
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
