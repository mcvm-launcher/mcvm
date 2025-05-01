import { Router, Route } from "@solidjs/router";
import "./App.css";
import LaunchPage from "./pages/launch/LaunchPage";
import NavBar from "./components/navigation/NavBar";
import { createSignal } from "solid-js";
import LaunchFooter from "./components/launch/LaunchFooter";
import InstanceConfig from "./pages/config/InstanceConfig";

export default function App() {
	const [selectedInstance, setSelectedInstance] = createSignal<string | null>(
		null
	);

	return (
		<Router
			root={({ children }) => (
				<Layout children={children} selectedInstance={selectedInstance()} />
			)}
		>
			<Route
				path="/"
				component={() => <LaunchPage onSelectInstance={setSelectedInstance} />}
			/>
			<Route path="/instance_config/:id" component={() => <InstanceConfig />} />
		</Router>
	);
}

function Layout(props: LayoutProps) {
	return (
		<>
			<NavBar />
			{props.children}
			<LaunchFooter selectedInstance={props.selectedInstance} />
		</>
	);
}

interface LayoutProps {
	selectedInstance: string | null;
	children: any;
}
