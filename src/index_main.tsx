import React from "react";
import ReactDOM from "react-dom/client";
import App from "./AppMain";
import register_launcher from "./plugins/launcher.tsx";
import register_js_host from "./plugins/js_host.tsx";
import { registry } from "./plugins/registry.tsx";
import { install_plugin_host } from "./plugins/host.ts";

import "./core.css"
import "./index_main.css"

// 内置插件与 JS 插件走同一条注册路径（见 src/plugins/registry.tsx）。
// 放在入口而不是某个组件的挂载副作用里：两个入口各挂一个 React app，
// 挂载在 StrictMode 下会跑两遍，而且配置窗口将来也要用同一份注册表。
register_launcher(registry);
// 插件条目是宿主自己画的一类行，宿主的那一条注册也在这里
register_js_host(registry);

// JS 插件宿主：从这里开始应 Rust 的两次请求。插件代码跑在各自的 Worker 里，不注入宿主文档
// （见 src/plugins/host.ts / plugin_worker.ts）
install_plugin_host();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
