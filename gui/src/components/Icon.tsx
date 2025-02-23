import { JSXElement } from "solid-js";

export default function Icon(props: IconProps) {
	return (
		<props.icon width={props.size} height={props.size} viewBox={`0 0 16 16`} />
	);
}

export interface IconProps {
	icon: (props: HasWidthHeight) => JSXElement;
	size: string;
}

export interface HasWidthHeight {
	width?: string;
	height?: string;
	viewBox?: string;
}
