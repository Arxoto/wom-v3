import type { PluginRegistry, PluginView } from "../registry.tsx";
import { IconHtmlPlugin, IconOpenPage } from "./icons";

export const HTML_PLUGIN_ITEM_TYPE = "html_plugin";

const HTML_HOST: PluginView = {
    id: "html_host",
    types: [
        {
            the_type: HTML_PLUGIN_ITEM_TYPE,
            actions: [{
                id: "open_html",
                label_key: "action.html_host.html_plugin.open_html",
                icon: <IconOpenPage></IconOpenPage>,
            }],
            icon: <IconHtmlPlugin></IconHtmlPlugin>,
        },
    ],
    labels: {
        "action.html_host.html_plugin.open_html": "打开页面",
    },
};

const register_html_host = (registry: PluginRegistry) => {
    registry.register(HTML_HOST);
}

export default register_html_host;
