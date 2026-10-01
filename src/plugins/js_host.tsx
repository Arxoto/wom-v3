import type { PluginRegistry, PluginView } from "./registry.tsx";
import { IconJsPlugin, IconOpenSearch } from "./js_host_icons";

/**
 * 插件条目的类型名：与 Rust 侧 `plugin_impl_js::JS_PLUGIN_ITEM_TYPE` 一字不差
 *
 * 接线层要据它认"当前这一行是不是插件条目"（Enter 与 ⇧+Enter 都进搜索页，spec §4.1），
 * 所以它是这一份注册里唯一被别处引用的东西。
 */
export const JS_PLUGIN_ITEM_TYPE = "js_plugin";

/**
 * JS 插件宿主自己的注册内容：插件条目长什么样
 *
 * 清单里没有"插件条目"这一条——`types` 是**结果行**的类型，而插件条目是宿主给每个
 * `Plugin Package` 画的那一行，所以由宿主自己注册（与 launcher 走同一个 `register`）。
 *
 * 类型名、动作 id 与文案键必须与 Rust 侧 `plugin_impl_js` 的常量一字不差：
 * `JS_PLUGIN_ITEM_TYPE` / `ACTION_OPEN_SEARCH` / `OPEN_SEARCH_LABEL_KEY`。
 * 对不上时界面只是不画图标、显示键本身，不会画错（见 `registry.tsx` 的查找）。
 */
const JS_HOST: PluginView = {
    id: "js_host",
    types: [
        {
            the_type: JS_PLUGIN_ITEM_TYPE,
            actions: [{
                id: "open_search",
                label_key: "action.js_host.js_plugin.open_search",
                icon: <IconOpenSearch></IconOpenSearch>,
            }],
            icon: <IconJsPlugin></IconJsPlugin>,
        },
    ],
    labels: {
        "action.js_host.js_plugin.open_search": "插件搜索",
    },
};

/** 把 JS 插件宿主的这一条注册进前端注册表 */
const register_js_host = (registry: PluginRegistry) => {
    registry.register(JS_HOST);
}

export default register_js_host;
