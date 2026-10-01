import type { PluginActionTable, PluginActionView, PluginItemDisplay } from "../../core";

/**
 * Item Action 的中文文案
 *
 * 键由 Rust 下发。同一个 `copy` 在 Scan 上是「复制完整路径」、在 Web 上是「复制链接」，
 * 所以键按类型分套。两套体系并行期间两套键都在这里：
 *
 * - 内建条目是三段式 `action.<类型>.<动作>`（对应 `builtin_plugins::action`）；
 * - launcher 插件是四段式 `action.<插件>.<类型>.<动作>`（对应 spec §2.3 / Q35）。
 *
 * 界面这一轮已经切到插件那一套，旧键留到迁移完成后再删。
 * 中文只住在这里：Rust 侧一个字都没有，键查不到就显示键本身。
 */
const LABELS: Record<string, string> = {
    // 内建条目（三段式）
    "action.snip.copy": "复制片段",
    "action.note.open_note": "打开笔记",
    "action.cmd.copy": "复制命令",
    "action.web.open_url": "打开链接",
    "action.web.copy": "复制链接",
    "action.scan.open_path": "默认打开",
    "action.scan.reveal": "在文件夹中选中",
    "action.scan.copy": "复制完整路径",

    // launcher 插件（四段式）
    "action.launcher.cmd.copy": "复制命令",
    "action.launcher.web.open_url": "打开链接",
    "action.launcher.web.copy": "复制链接",
    "action.launcher.scan.open_path": "默认打开",
    "action.launcher.scan.reveal": "在文件夹中选中",
    "action.launcher.scan.copy": "复制完整路径",
};

/** 文案键对应的中文；查不到就显示键本身 */
export const action_label = (label_key: string): string => LABELS[label_key] ?? label_key;

/**
 * 某个条目支持的动作
 *
 * 顺序取自条目自带的 `action_ids`（见 spec §1.5：条目携带有序动作列表，第一个是默认动作），
 * `label_key` 从动作表里按类型与动作 id 查——查不到就退回动作 id 本身，于是文案位置显示的是 id。
 */
export const actions_of = (
    type_actions: PluginActionTable,
    item: PluginItemDisplay,
): PluginActionView[] =>
    item.action_ids.map(id => ({
        id,
        label_key: type_actions[item.the_type]?.find(action => action.id === id)?.label_key ?? id,
    }));

/**
 * 条目当前动作：按下标取，下标越界时夹到最后一个
 *
 * 下标是每个条目自己记的（见 `MainState.action_indices`），越界只可能来自动作表本身的变化。
 */
export const current_action = (
    type_actions: PluginActionTable,
    item: PluginItemDisplay,
    action_index: number,
): PluginActionView | null => {
    const actions = actions_of(type_actions, item);
    return actions[Math.min(action_index, actions.length - 1)] ?? null;
}

/** 条目当前动作的文案；没有条目、或它没有动作时为 `null`（调用方整块不渲染） */
export const current_action_label = (
    type_actions: PluginActionTable,
    item: PluginItemDisplay | undefined,
    action_index: number,
): string | null => {
    if (!item) return null;
    const action = current_action(type_actions, item, action_index);
    return action ? action_label(action.label_key) : null;
}
