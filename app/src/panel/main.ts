import { mount } from "svelte";
import "../styles/base.css";
import "./panel.css";
import { setupWindow } from "../lib/platform";
import Panel from "./Panel.svelte";

setupWindow({ vibrancy: true });
export default mount(Panel, { target: document.getElementById("app")! });
