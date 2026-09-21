import type { ItemAction, ItemDisplay, ItemType, ItemTypeActions } from "../../core";

/**
 * Item Type + Item Action 的中文文案
 *
 * 键由 Rust 下发（对应 Rust 侧 `action::ItemAction::label_key`）。同一个 `copy`
 * 在 `File` 上是「复制完整路径」、在 `Web` 上是「复制链接」，所以键按 ItemType 分套。
 *
 * 中文只住在这里：Rust 侧一个字都没有，键查不到就显示键本身。
 */
const LABELS: Record<string, string> = {
    "action.snip.copy": "复制片段",
    "action.note.open_note": "打开笔记",
    "action.cmd.copy": "复制命令",
    "action.web.open_url": "打开链接",
    "action.web.copy": "复制链接",
    "action.file.open_path": "默认打开",
    "action.file.reveal": "在文件夹中选中",
    "action.file.copy": "复制完整路径",
    "action.scan.open_path": "默认打开",
    "action.scan.reveal": "在文件夹中选中",
    "action.scan.copy": "复制完整路径",
};

/** 文案键对应的中文；查不到就显示键本身 */
export const action_label = (label_key: string): string => LABELS[label_key] ?? label_key;

/** 某个类型的动作表；`ItemTypeActions` 是七个键齐全的 `Record`，取出来必定有值 */
export const actions_of = (type_actions: ItemTypeActions, the_type: ItemType): ItemAction[] =>
    type_actions[the_type];

/**
 * 条目当前动作：按下标取，下标越界时夹到最后一个
 *
 * 下标是每个条目自己记的（见 `MainState.action_indices`），越界只可能来自动作表本身的变化。
 */
export const current_action = (
    type_actions: ItemTypeActions,
    the_type: ItemType,
    action_index: number,
): ItemAction | null => {
    const actions = actions_of(type_actions, the_type);
    return actions[Math.min(action_index, actions.length - 1)] ?? null;
}

/** 条目当前动作的文案；没有条目、或它没有动作时为 `null`（调用方整块不渲染） */
export const current_action_label = (
    type_actions: ItemTypeActions,
    item: ItemDisplay | undefined,
    action_index: number,
): string | null => {
    if (!item) return null;
    const action = current_action(type_actions, item.the_type, action_index);
    return action ? action_label(action.label_key) : null;
}
