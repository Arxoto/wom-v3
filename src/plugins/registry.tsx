import type { ReactNode } from "react";

import type { PluginItemDisplay, PluginSearchRow } from "../core";

/**
 * 前端插件注册表
 *
 * 插件在 Rust 侧注册条目与动作，到了界面这一层还剩三件只有前端知道的事：
 * 类型名画哪张图标、动作 id 画哪张图标、label_key 是什么中文。这三张表就住在这里，
 * 键一律是**不透明字符串**——Rust 侧从来不解释类型名与动作 id（见 spec §1.2 / §1.5），
 * 前端认不出就不画，而不是碰巧落到别的类型上。
 *
 * 内置的 Rust 插件与将来的 JS 插件走**同一条注册路径**（spec §五 / Q39）：
 * 内置插件的默认导出就是一次注册调用（见 `launcher.tsx`），JS 插件将来调同一个
 * `register`。两条路径分开的话，JS 插件能用的能力会永远只是内置能力的子集。
 *
 * JS 插件拿到这个单例的挂载点就是 `host.ts`：宿主往 `window.__WOM_PLUGIN__` 挂一个窄接口，
 * 里面的 `register` 转调这里的 `register`（见 docs/adr/0011）。**挂的是接口，不是注册表本身**。
 * 本文件是纯数据，没有 React 组件；`.tsx` 只是因为注册进来的图标可以是 JSX 元素。
 */

/**
 * 一张图标
 *
 * 内置插件（launcher）交上来的是 JSX 元素，JS 插件交上来的只能是一段字符串——
 * 它在 webview 里跑，但手上没有 React（见 `host.ts` 的 `PluginRegistration`）。
 * 字符串按图片地址处理（data URI 也算），由 `PluginIcon` 画成 `<img>`。
 */
export type PluginIcon = ReactNode | string;

/** 一条动作的渲染面：动作 id、文案键、图标（认不出就不给） */
export interface PluginActionView {
    id: string,
    label_key: string,
    icon: PluginIcon | null,
}

/** 一个插件注册的类型：类型名、它的动作表（顺序即优先级，第一个是默认动作）、条目图标 */
export interface PluginTypeView {
    the_type: string,
    actions: PluginActionView[],
    icon: PluginIcon | null,
}

/**
 * 一个插件注册进来的全部内容
 *
 * `id` 是插件的稳定 ASCII 标识，与 Rust 侧 `PluginId` 同一个值；`types` 的顺序即注册顺序。
 * `labels` 是这个插件的 `label_key` → 中文，文案**跟着插件走**：同一个 `copy`
 * 在 Scan 上是「复制完整路径」、在 Web 上是「复制链接」，Rust 侧分套的正是这个要求。
 */
export interface PluginView {
    id: string,
    types: PluginTypeView[],
    labels: Record<string, string>,
}

/** 插件自己的搜索函数：给关键字，还它要画的那几行（形状由 `normalize` 兜住） */
export type PluginSearchFn = (keyword: string) => unknown;

/** 结果行动作的处理函数：要让插件干什么由它自己决定，宿主只看它有没有抛错 */
export type PluginRunFn = (row: PluginSearchRow | undefined, action_id: string) => void;

/**
 * 一次注册的全部内容：界面的那几张表 + 插件自己的两个函数
 *
 * `search` / `run` 是 JS 插件才有的（内置插件在 Rust 侧做这些事），
 * 它们不进 `PluginView`：界面的表里只该有画一行所需的东西。
 */
export interface PluginRegistration extends PluginView {
    search?: PluginSearchFn,
    run?: PluginRunFn,
}

/**
 * 插件注册表的对外形状
 *
 * 界面只读：取条目的动作、取条目当前的动作与文案、取图标与动作表。写只有 `register` 一处。
 */
export interface PluginRegistry {
    /** 注册一个插件；同一个 `id` 重复注册不再生效（挂载期重复调用是安全的） */
    register(plugin: PluginRegistration): void;
    /** 这个 `id` 注册过没有：装载器判断「已注册」用的就是它 */
    has(plugin_id: string): boolean;
    /** 取插件自己的搜索函数；内置插件没有这个 */
    search_of(plugin_id: string): PluginSearchFn | undefined;
    /** 取结果行动作的处理函数；内置插件没有这个 */
    run_of(plugin_id: string): PluginRunFn | undefined;
    /** 该条目支持的动作；顺序取自条目自带的 `action_ids`（见 spec §1.5） */
    actions_of(item: PluginItemDisplay): PluginActionView[];
    /** 条目当前动作：按下标取，下标越界时夹到最后一个；没有动作时为 `null` */
    current_action(item: PluginItemDisplay, action_index: number): PluginActionView | null;
    /** 条目当前动作的中文文案；没有条目、或它没有动作时为 `null`（调用方整块不渲染） */
    current_action_label(item: PluginItemDisplay | undefined, action_index: number): string | null;
    /** 按类型名取条目图标；认不出为 `null` */
    type_icon(the_type: string): PluginIcon | null;
    /** 该类型全部已注册的动作；认不出的类型是空表 */
    actions_of_type(the_type: string): PluginActionView[];
    /**
     * 按动作 id 取动作图标；认不出为 `null`
     *
     * 行里只带动作 id，不带它属于哪个插件，所以这是**全表**查：两个插件用同一个动作 id
     * 配不同的图标时会撞在一起。动作 id 目前由插件各自定义、跨插件并不保证唯一
     * （`open_url` / `copy` 这类通用名尤其），等真有 JS 插件注册进来再谈要不要加插件维度。
     */
    action_icon(action_id: string): PluginIcon | null;
}

/** 认不出的类型：空表，界面因此不画动作、也不画动作图标 */
const NO_ACTIONS: PluginActionView[] = [];

class Registry implements PluginRegistry {
    private plugins: PluginView[] = [];
    private registered_ids = new Set<string>();
    /** 所有插件交上来的文案表；一个键在同一张表里只该有一条中文 */
    private labels: Record<string, string> = {};
    /** 插件自己的函数：只有 JS 插件有，界面不读它 */
    private runtime = new Map<string, { search?: PluginSearchFn, run?: PluginRunFn }>();

    register(plugin: PluginRegistration): void {
        if (this.registered_ids.has(plugin.id)) return;
        this.registered_ids.add(plugin.id);
        // 只留画行要用的三样：`search` / `run` 是宿主的账，不进界面读的那张表
        this.plugins.push({ id: plugin.id, types: plugin.types, labels: plugin.labels });
        Object.assign(this.labels, plugin.labels);

        if (plugin.search || plugin.run) {
            this.runtime.set(plugin.id, { search: plugin.search, run: plugin.run });
        }
    }

    has(plugin_id: string): boolean {
        return this.registered_ids.has(plugin_id);
    }

    search_of(plugin_id: string): PluginSearchFn | undefined {
        return this.runtime.get(plugin_id)?.search;
    }

    run_of(plugin_id: string): PluginRunFn | undefined {
        return this.runtime.get(plugin_id)?.run;
    }

    actions_of_type(the_type: string): PluginActionView[] {
        for (const plugin of this.plugins) {
            const type = plugin.types.find(t => t.the_type === the_type);
            if (type) return type.actions;
        }
        return NO_ACTIONS;
    }

    type_icon(the_type: string): PluginIcon | null {
        for (const plugin of this.plugins) {
            const type = plugin.types.find(t => t.the_type === the_type);
            if (type) return type.icon;
        }
        return null;
    }

    action_icon(action_id: string): PluginIcon | null {
        for (const plugin of this.plugins) {
            for (const type of plugin.types) {
                const action = type.actions.find(a => a.id === action_id);
                if (action) return action.icon;
            }
        }
        return null;
    }

    actions_of(item: PluginItemDisplay): PluginActionView[] {
        const actions = this.actions_of_type(item.the_type);
        // 动作 id 在动作表里查不到就退回 id 本身：动作位置显示的是 id，而不是空白
        return item.action_ids.map(id =>
            actions.find(action => action.id === id) ?? { id, label_key: id, icon: null });
    }

    current_action(item: PluginItemDisplay, action_index: number): PluginActionView | null {
        const actions = this.actions_of(item);
        return actions[Math.min(action_index, actions.length - 1)] ?? null;
    }

    current_action_label(item: PluginItemDisplay | undefined, action_index: number): string | null {
        if (!item) return null;
        const action = this.current_action(item, action_index);
        // 中文只住在前端：Rust 只下发 `label_key`，查不到键就显示键本身
        return action ? this.labels[action.label_key] ?? action.label_key : null;
    }
}

/**
 * 全局唯一的注册表
 *
 * 两个 HTML 入口各挂一个 React app，所以这个单例是**每个 webview 一份**，
 * 不是进程一份；它只装前端知道的那几张表，条目本身在 Rust 侧。
 */
export const registry = new Registry();
