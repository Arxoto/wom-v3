import type { PluginRegistry, PluginView } from "../registry.tsx";
import { IconCmd, IconScan, IconSys, IconWeb } from "./launcher_icons";
import { IconCopy, IconOpenPath, IconOpenUrl, IconReveal } from "./launcher_action_icons";

/**
 * launcher 插件的前端适配
 *
 * Rust 侧的 launcher 插件（`plugin_impl_launcher`）注册条目与动作，这里补上界面才知道的三件事：
 * 四个类型各自的图标、四个动作各自的图标、以及动作的中文文案。类型名与动作 id 与 Rust 侧
 * 一字不差，改一边就得改另一边；类型名对不上时界面只是不画图标，不会画错。
 *
 * 这份注册与将来的 JS 插件调的是同一个 `register`（spec §五 / Q39）：
 * `launcher` 这个 id 与 Rust 侧 `LAUNCHER_PLUGIN_ID` 是同一个值
 * （`plugin_impl_launcher/action.rs`）。
 */

/** label_key 的中文；键是四段式 `action.<插件>.<类型>.<动作>`（spec §2.3 / Q35） */
const ACTION_LABELS: Record<string, string> = {
    "action.launcher.cmd.copy": "复制命令",
    "action.launcher.web.open_url": "打开链接",
    "action.launcher.web.copy": "复制链接",
    "action.launcher.scan.open_path": "默认打开",
    "action.launcher.scan.reveal": "在文件夹中选中",
    "action.launcher.scan.copy": "复制完整路径",
};

/**
 * launcher 注册进前端注册表的全部内容
 *
 * 顺序即优先级，第一个是默认动作；这与 Rust 侧 `action.rs` 里的动作表、以及
 * manifest 里条目的 `action_ids` 必须一致，否则默认动作会与后端不一致。
 */
const LAUNCHER: PluginView = {
    id: "launcher",
    types: [
        // Sys 没有动作：每个系统命令的动作就是它的描述内容
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

/**
 * 把 launcher 注册进前端注册表
 *
 * 这是内置插件的**注册入口**：内置的那一份走的就是这条调用，与 JS 插件将来走的是同一个。
 * 同一个 id 重复注册不再生效（见 `registry.tsx`），所以挂载期重复调用是安全的。
 */
const register_launcher = (registry: PluginRegistry) => {
    registry.register(LAUNCHER);
}

export default register_launcher;
