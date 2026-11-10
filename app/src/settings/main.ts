import { mount } from "svelte";
import "../styles/base.css";
import "./settings.css";
import { setupWindow } from "../lib/platform";
import Settings from "./Settings.svelte";

setupWindow({ vibrancy: false });
export default mount(Settings, { target: document.getElementById("app")! });
