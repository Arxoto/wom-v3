import { convertFileSrc, invoke } from "@tauri-apps/api/core";

import "./index_iframe.css";

const page = document.getElementById("plugin-page") as HTMLIFrameElement | null;

const show_plugin_page = async () => {
    const path = await invoke<string>("plugin_window_page");
    if (page) page.src = convertFileSrc(path);
}

void show_plugin_page();
