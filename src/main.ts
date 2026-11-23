// The frontend entry point.

import { mount } from "svelte";
import App from "./app/App.svelte";
import "./styles/app.css";

const target = document.getElementById("app");
if (!target) {
  throw new Error("the #app mount point is missing from index.html");
}

export default mount(App, { target });
