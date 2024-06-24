import "./LaunchPage.css";
import LaunchInstanceList from "../../components/launch/LaunchInstanceList";

export default function LaunchPage(props: LaunchPageProps) {
	return (
		<div class="container">
			{/* <h1 class="noselect">Launch</h1> */}
			<br />
			<LaunchInstanceList onSelectInstance={props.onSelectInstance} />
			<br />
		</div>
	);
}

export interface LaunchPageProps {
	onSelectInstance: (instance: string) => void;
}
