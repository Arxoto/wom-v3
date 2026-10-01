import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { info, warn } from "@tauri-apps/plugin-log";

import { plugin_report_search_results, type PluginSearchRow } from "../core";
import { registry, type PluginRegistration } from "./registry.tsx";

/**
 * JS 插件宿主的前端一半：装载插件、把窄接口挂到 `window` 上、应 Rust 的两次请求
 *
 * 另一半在 Rust 侧（`plugin_impl_js` + `plugin_host`）：清单扫描、框架里的代理、投影。
 * 两边共用同一个 `PluginId`（清单的 `id`），合起来才是一个完整的插件。
 *
 * 为什么是"注入 `<script src>`"而不是 `eval`：见 docs/adr/0011。
 * 代价是插件跑在宿主自己的文档与全局里，这一轮明确接受（spec §3.5）。
 */

/** 宿主请前端跑一次插件搜索（对应 Rust 侧 `constants::EVENT_PLUGIN_SEARCH_REQUEST`） */
const EVENT_SEARCH_REQUEST = "plugin_search_request";

/** 宿主把结果行的一个动作送回插件（对应 Rust 侧 `constants::EVENT_PLUGIN_ACTION_REQUEST`） */
const EVENT_ACTION_REQUEST = "plugin_action_request";

/** 去程的载荷（对应 Rust 侧 `plugin_impl_js::SearchRequest`） */
interface SearchRequestPayload {
    plugin_id: string,
    entry: string,
    keyword: string,
}

/** 动作请求的载荷（对应 Rust 侧 `plugin_impl_js::ActionRequest`） */
interface ActionRequestPayload {
    plugin_id: string,
    /** 行在最近一次搜索结果里的下标：账本就是本文件报上去的那一份 */
    row_id: number,
    action_id: string,
}

/**
 * 宿主挂给插件的那一个窄接口（spec §3.2）
 *
 * 只挂这一个对象，**不挂注册表本身**：插件能做的事就是注册自己、记日志、报错。
 * 名字带 `__WOM_PLUGIN__` 前缀，免得与将来的自定义前端页面撞车。
 */
export interface PluginHostApi {
    /** 交出这个插件的类型、动作图标与文案表，以及它自己的搜索函数与结果行动作处理函数 */
    register(spec: unknown): void;
    log(text: unknown): void;
    fail(text: unknown): void;
}

declare global {
    interface Window {
        __WOM_PLUGIN__?: PluginHostApi;
    }
}

/**
 * 装载结果记账：同一个插件只装一次
 *
 * 成功留在表里；失败**删掉**，下次再进这个插件的搜索页时重试——
 * 脚本坏了不该把重试也一起锁死到退出应用。
 */
const loaded = new Map<string, Promise<boolean>>();

/**
 * 最近一次报给 Rust 的结果行
 *
 * Rust 按行在数组里的下标（`local_id`）派发动作，所以要留一份一模一样的：
 * **归一化时不丢行**（见 `normalize_rows`），否则后面的行会全部错位。
 */
const reported = new Map<string, PluginSearchRow[]>();

/**
 * 装上宿主：挂全局窄接口、开始应 Rust 的请求
 *
 * 在渲染之前调用（见 `index_main.tsx`）：插件脚本要能在装载时立刻调 `register`，
 * 而这个接口必须已经在 `window` 上。
 */
export const install_plugin_host = () => {
    window.__WOM_PLUGIN__ = {
        register: spec => {
            if (!is_registration(spec)) {
                void warn("[plugin] invalid register spec, dropped");
                return;
            }
            registry.register(spec);
        },
        log: text => void info(`[plugin] ${String(text)}`),
        fail: text => void warn(`[plugin] ${String(text)}`),
    };

    void listen<SearchRequestPayload>(EVENT_SEARCH_REQUEST, event => {
        void on_search_request(event.payload);
    });
    void listen<ActionRequestPayload>(EVENT_ACTION_REQUEST, event => {
        on_action_request(event.payload);
    });
};

/**
 * 注册内容的最低要求
 *
 * 这一段是**从 webview 来的不可信数据**：字段缺了、类型不对，一律整份丢掉并记一条日志，
 * 宁可这个插件什么都不画，也别让半份注册把界面带坏。
 */
const is_registration = (spec: unknown): spec is PluginRegistration => {
    if (typeof spec !== "object" || spec === null) return false;

    const candidate = spec as Partial<PluginRegistration>;

    return (
        typeof candidate.id === "string"
        && candidate.id !== ""
        && Array.isArray(candidate.types)
        && typeof candidate.labels === "object"
        && candidate.labels !== null
    );
};

/**
 * 装载一个插件，返回它有没有装好
 *
 * 装载完成是一个 Promise，插件的搜索**串在它后面**——这就是"先等已注册、再发起搜索"
 * 那条硬要求（spec §2.4 / §3.3）：图标与文案必须在结果行渲染之前到位。
 */
const ensure_loaded = (plugin_id: string, entry: string): Promise<boolean> => {
    const cached = loaded.get(plugin_id);
    if (cached !== undefined) return cached;

    const loading = inject_plugin_script(plugin_id, entry).then(ok => {
        // 失败不记账：下一次进入这个插件的搜索页时重试
        if (!ok) loaded.delete(plugin_id);
        return ok;
    });

    loaded.set(plugin_id, loading);

    return loading;
};

/**
 * 把插件的 JS 入口注入 webview
 *
 * 取文件与执行都是 webview 自己的行为：宿主从头到尾不碰插件源码字符串（ADR-0011）。
 * 路径由 Rust 给出（`SearchRequest.entry`），`convertFileSrc` 转成 asset URL，
 * 再由 asset protocol 的 scope 决定放不放行。
 */
const inject_plugin_script = (plugin_id: string, entry: string): Promise<boolean> => {
    const url = convertFileSrc(entry);

    return new Promise<boolean>(resolve => {
        const script = document.createElement("script");
        script.src = url;

        script.onload = () => {
            script.remove();
            // 「已注册」的判据是插件真的调了 register，而不是脚本文件到手了
            const registered = registry.has(plugin_id);
            if (!registered) void warn(`[plugin] script loaded but registered nothing: ${plugin_id}`);
            resolve(registered);
        };

        script.onerror = () => {
            script.remove();
            // 走到这里多半是 asset protocol 没放行这个路径，或者文件根本不在
            void warn(`[plugin] script load failed: ${plugin_id} ${url}`);
            resolve(false);
        };

        document.head.appendChild(script);
    });
};

/**
 * 跑一次 `Plugin Search`：装载 → 插件自己的搜索函数 → 归一化
 *
 * 任何一步失败都只是"这一次搜索没有结果"：结果页显示成空页，主窗口照常（spec §4.4）。
 */
const on_search_request = async (payload: SearchRequestPayload) => {
    const rows = await search_rows(payload);

    reported.set(payload.plugin_id, rows);

    try {
        await plugin_report_search_results(payload.plugin_id, rows);
    } catch (err) {
        // 回程失败说明这一次搜索已经过期（用户退出了搜索页）：记一条就够，不重试
        void warn(`[plugin] report search results failed: ${String(err)}`);
    }
};

/**
 * 装载并调用插件自己的搜索函数
 *
 * **绝不抛错**：Rust 那一头在等回程，这里漏一个异常出去，那次搜索就永远不落地。
 * 任何一步出问题都收成"这一次没有结果"。
 */
const search_rows = async (payload: SearchRequestPayload): Promise<PluginSearchRow[]> => {
    const { plugin_id, entry, keyword } = payload;

    try {
        if (!await ensure_loaded(plugin_id, entry)) return [];

        const search = registry.search_of(plugin_id);
        if (search === undefined) {
            void warn(`[plugin] no search function registered: ${plugin_id}`);
            return [];
        }

        return normalize_rows(await search(keyword));
    } catch (err) {
        // 插件代码抛错、asset URL 拼不出来、脚本注入失败……都在这里收住
        void warn(`[plugin] search failed: ${plugin_id} ${String(err)}`);
        return [];
    }
};

/**
 * 把插件交出来的东西收成结果行的形状
 *
 * **一行都不丢**：Rust 按行在数组里的下标派发动作，这里少一行，后面的行就全部错位。
 * 字段认不出来就是空串 / 空表——那一行画不出图标、点不出动作，但它还在原来的位置上。
 */
const normalize_rows = (raw: unknown): PluginSearchRow[] => {
    if (!Array.isArray(raw)) return [];

    return raw.map(row => {
        const source = (typeof row === "object" && row !== null ? row : {}) as Record<string, unknown>;
        const action_ids = Array.isArray(source.action_ids)
            ? source.action_ids.filter((id): id is string => typeof id === "string")
            : [];

        return {
            the_type: typeof source.the_type === "string" ? source.the_type : "",
            name: typeof source.name === "string" ? source.name : "",
            desc: typeof source.desc === "string" ? source.desc : "",
            action_ids,
        };
    });
};

/**
 * 结果行的一个动作：交回插件自己跑
 *
 * 这条路是单向的（`Plugin::run_action` 是同步的，见 `plugin_impl_js`），所以插件抛错
 * 只在这里记一条日志——Rust 那边已经按"派发过了"处理了。
 */
const on_action_request = (payload: ActionRequestPayload) => {
    const row = reported.get(payload.plugin_id)?.[payload.row_id];

    if (row === undefined) {
        void warn(`[plugin] action on an unknown row: ${payload.plugin_id} ${payload.row_id}`);
        return;
    }

    const run = registry.run_of(payload.plugin_id);
    if (run === undefined) {
        void warn(`[plugin] no action handler registered: ${payload.plugin_id}`);
        return;
    }

    try {
        run(row, payload.action_id);
        void info(`[plugin] action dispatched: ${payload.plugin_id} ${payload.action_id}`);
    } catch (err) {
        void warn(`[plugin] action failed: ${payload.plugin_id} ${String(err)}`);
    }
};
