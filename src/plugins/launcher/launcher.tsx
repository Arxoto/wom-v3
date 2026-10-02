import type { PluginRegistry, PluginView } from "../registry.tsx";
import { IconCmd, IconScan, IconSys, IconWeb } from "./launcher_icons";
import { IconCopy, IconOpenPath, IconOpenUrl, IconReveal } from "./launcher_action_icons";

const ACTION_LABELS: Record<string, string> = {
    "action.launcher.cmd.copy": "复制命令",
    "action.launcher.web.open_url": "打开链接",
    "action.launcher.web.copy": "复制链接",
    "action.launcher.scan.open_path": "默认打开",
    "action.launcher.scan.reveal": "在文件夹中选中",
    "action.launcher.scan.copy": "复制完整路径",
};

const LAUNCHER: PluginView = {
    id: "launcher",
    types: [
        { the_type: "sys", actions: [], icon: <IconSys></IconSys> },
        {
            the_type: "cmd",
            actions: [{ id: "copy", label_key: "action.launcher.cmd.copy", icon: <IconCopy></IconCopy> }],
            icon: <IconCmd></IconCmd>,
        },
        {
            the_type: "web",
            actions: [
                { id: "open_url", label_key: "action.launcher.web.open_url", icon: <IconOpenUrl></IconOpenUrl> },
                { id: "copy", label_key: "action.launcher.web.copy", icon: <IconCopy></IconCopy> },
            ],
            icon: <IconWeb></IconWeb>,
        },
        {
            the_type: "scan",
            actions: [
                { id: "open_path", label_key: "action.launcher.scan.open_path", icon: <IconOpenPath></IconOpenPath> },
                { id: "reveal", label_key: "action.launcher.scan.reveal", icon: <IconReveal></IconReveal> },
                { id: "copy", label_key: "action.launcher.scan.copy", icon: <IconCopy></IconCopy> },
            ],
            icon: <IconScan></IconScan>,
        },
    ],
    labels: ACTION_LABELS,
};

const register_launcher = (registry: PluginRegistry) => {
    registry.register(LAUNCHER);
}

export default register_launcher;
