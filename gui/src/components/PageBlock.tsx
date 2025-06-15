export default function PageBlock(props: PageBlockProps) {
	return (
		<div
			id="pageblock"
			style="position:fixed; width:100vw; height:100vh; left:0px; top:0px; background-color:#050505; opacity:0.7;z-index:10"
			onclick={props.onClick}
		></div>
	);
}

export interface PageBlockProps {
	onClick?: () => void;
}
