import type { PluginRegistry, PluginView } from "../registry.tsx";
import { IconJsPlugin, IconOpenSearch } from "./icons";

export const JS_PLUGIN_ITEM_TYPE = "js_plugin";

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

const register_js_host = (registry: PluginRegistry) => {
    registry.register(JS_HOST);
}

export default register_js_host;
