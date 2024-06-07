import { Router, Route } from "@solidjs/router";
import "./App.css";
import LaunchPage from "./pages/launch/LaunchPage";
import NavBar from "./components/navigation/NavBar";

export default function App() {
	return (
		<Router root={Layout}>
			<Route path="/" component={LaunchPage} />
		</Router>
	);
}

function Layout(props: any) {
	return (
		<>
			<NavBar />
			{props.children}
		</>
	);
}
