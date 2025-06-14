import { JSXElement } from "solid-js";

export default function Icon(props: IconProps) {
	return (
		<props.icon
			width={props.size}
			height={props.size}
			viewBox={`0 0 16 16`}
			{...props}
		/>
	);
}

export interface IconProps {
	icon: (props: HasWidthHeight) => JSXElement;
	size: string;
	[prop: string]: any;
}

export interface HasWidthHeight {
	width?: string;
	height?: string;
	viewBox?: string;
	[prop: string]: any;
}
