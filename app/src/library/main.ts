import { mount } from "svelte";
import "../styles/base.css";
import { setupWindow } from "../lib/platform";
import Library from "./Library.svelte";

setupWindow({ vibrancy: true });
export default mount(Library, { target: document.getElementById("app")! });
