import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { info, warn } from "@tauri-apps/plugin-log";

import { plugin_report_search_results, type PluginSearchRow } from "../core";
import { registry, type PluginView } from "./registry.tsx";

/**
 * JS 插件宿主的前端一半：给每个插件起一个 Worker、应 Rust 的两次请求
 *
 * 另一半在 Rust 侧（`plugin_impl_js` + `plugin_host`）：清单扫描、框架里的代理、投影。
 * 两边共用同一个 `PluginId`（清单的 `id`），合起来才是一个完整的插件。
 *
 * 插件代码跑在 Worker 里（`plugin_worker.ts`），与宿主文档隔开；本文件只负责
 * 「起 Worker → 发入口 → 转发注册与结果行 → 把动作送回去」这一条线。
 * 入口怎么变成 URL、为什么是经典 Worker，见 `plugin_worker.ts` 的文件头。
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
 * 最近一次报给 Rust 的结果行
 *
 * Rust 按行在数组里的下标（`local_id`）派发动作，所以要留一份一模一样的：
 * **归一化时不丢行**（在 Worker 里做），否则后面的行会全部错位。
 */
const reported = new Map<string, PluginSearchRow[]>();

/**
 * 装上宿主：开始应 Rust 的两次请求
 *
 * 在渲染之前调用（见 `index_main.tsx`）。插件代码不再注入宿主文档，所以这里不挂全局接口；
 * 插件能看到的 `__WOM_PLUGIN__` 由 `plugin_worker.ts` 挂在 Worker 自己的全局上。
 */
export const install_plugin_host = () => {
    void listen<SearchRequestPayload>(EVENT_SEARCH_REQUEST, event => {
        void on_search_request(event.payload);
    });
    void listen<ActionRequestPayload>(EVENT_ACTION_REQUEST, event => {
        void on_action_request(event.payload);
    });
};

/**
 * 一个插件包的执行侧：一个 Worker，加上宿主这边的请求账本
 *
 * 一个包一个 Worker：插件之间不共享全局，取消时 `terminate()` 就够——"重载插件代码"
 * 从此有了着力点，下一步把重扫接上来即可。
 */
class PluginRuntime {
    private readonly plugin_id: string;
    private readonly worker: Worker;
    private next_request_id = 1;
    /** 在飞的装载请求：Worker 自己都起不来时，靠它把这一次装载收成失败而不是永等 */
    private load_request_id: number | null = null;
    private readonly pending = new Map<number, (message: PluginWire.WorkerToHost | null) => void>();

    private constructor(plugin_id: string, worker: Worker) {
        this.plugin_id = plugin_id;
        this.worker = worker;
        this.worker.onmessage = event => this.on_message(event.data as PluginWire.WorkerToHost);
        this.worker.onerror = event => this.on_error(event);
    }

    /**
     * 起一个 Worker 并装载入口
     *
     * 成功判据沿用旧版：**注册了才算装好**，脚本到手不算。失败返回 `null`，调用方下次重试。
     */
    static async create(plugin_id: string, entry: string): Promise<PluginRuntime | null> {
        // 经典 Worker：`importScripts` 取 asset URL 不受 CORS 限制，见 `plugin_worker.ts`
        const worker = new Worker(new URL("./plugin_worker.ts", import.meta.url), { type: "classic" });
        const runtime = new PluginRuntime(plugin_id, worker);

        const loaded = await runtime.load(convertFileSrc(entry));

        if (!loaded.ok || !registry.has(plugin_id)) {
            void warn(`[plugin] load failed: ${plugin_id} ${loaded.reason || "registered nothing"}`);
            runtime.dispose();
            return null;
        }

        return runtime;
    }

    private take_id(): number {
        return this.next_request_id++;
    }

    private send(message: Extract<PluginWire.HostToWorker, { request_id: number }>): Promise<PluginWire.WorkerToHost | null> {
        return new Promise(resolve => {
            this.pending.set(message.request_id, resolve);
            this.worker.postMessage(message);
        });
    }

    private async load(entry_url: string): Promise<{ ok: boolean, reason: string }> {
        const request_id = this.take_id();
        this.load_request_id = request_id;

        const message = await this.send({ kind: "load", request_id, entry_url });
        this.load_request_id = null;

        if (message === null || message.kind !== "loaded") {
            return { ok: false, reason: "worker failed before load finished" };
        }

        return { ok: message.ok, reason: message.reason };
    }

    async search(keyword: string): Promise<PluginSearchRow[]> {
        const request_id = this.take_id();
        const message = await this.send({ kind: "search", request_id, keyword });

        if (message === null || message.kind !== "rows") {
            void warn(`[plugin] search request dropped: ${this.plugin_id}`);
            return [];
        }

        return message.rows;
    }

    /** 结果行的动作：单向送回去，不等回程（Rust 那边也是这么派发的） */
    run(row: PluginSearchRow, action_id: string): void {
        const message: PluginWire.HostToWorker = { kind: "run", row, action_id };
        this.worker.postMessage(message);
    }

    dispose(): void {
        this.worker.terminate();
        this.fail_pending("runtime disposed");
    }

    private fail_pending(reason: string): void {
        if (this.pending.size === 0) return;

        void warn(`[plugin] pending requests dropped: ${this.plugin_id} (${reason})`);
        for (const resolve of this.pending.values()) resolve(null);
        this.pending.clear();
    }

    private on_message(message: PluginWire.WorkerToHost): void {
        switch (message.kind) {
            case "register":
                // 这一段是**从 Worker 来的不可信数据**：字段缺了、类型不对，整份丢掉
                if (is_registration(message.spec)) {
                    registry.register(message.spec);
                } else {
                    void warn(`[plugin] invalid register spec, dropped: ${this.plugin_id}`);
                }
                return;
            case "log":
                void info(`[plugin] ${message.text}`);
                return;
            case "fail":
                void warn(`[plugin] ${message.text}`);
                return;
            case "loaded":
            case "rows": {
                const resolve = this.pending.get(message.request_id);
                this.pending.delete(message.request_id);
                resolve?.(message);
                return;
            }
        }
    }

    private on_error(event: ErrorEvent): void {
        void warn(`[plugin] worker error: ${this.plugin_id} ${event.message}`);

        // 装载期出错（Worker 脚本本身取不到之类）时把这一次装载收掉，别让调用方永远等下去
        if (this.load_request_id !== null) {
            const resolve = this.pending.get(this.load_request_id);
            this.pending.delete(this.load_request_id);
            resolve?.(null);
        }
    }
}

/**
 * 装载结果记账：同一个插件只装一次
 *
 * 成功留在表里；失败**删掉**，下次再进这个插件的搜索页时重试——
 * 插件坏了不该把重试也一起锁死到退出应用。
 */
const loaded = new Map<string, Promise<PluginRuntime | null>>();

/**
 * 注册内容的最低要求
 *
 * 这一段是**从 Worker 来的不可信数据**：字段缺了、类型不对，一律整份丢掉并记一条日志，
 * 宁可这个插件什么都不画，也别让半份注册把界面带坏。
 */
const is_registration = (spec: unknown): spec is PluginView => {
    if (typeof spec !== "object" || spec === null) return false;

    const candidate = spec as Partial<PluginView>;

    return (
        typeof candidate.id === "string"
        && candidate.id !== ""
        && Array.isArray(candidate.types)
        && typeof candidate.labels === "object"
        && candidate.labels !== null
    );
};

/**
 * 装载一个插件，返回它的执行侧
 *
 * 装载完成是一个 Promise，插件的搜索**串在它后面**——这就是"先等已注册、再发起搜索"
 * 那条硬要求（spec §2.4 / §3.3）：图标与文案必须在结果行渲染之前到位。
 */
const ensure_loaded = (plugin_id: string, entry: string): Promise<PluginRuntime | null> => {
    const cached = loaded.get(plugin_id);
    if (cached !== undefined) return cached;

    const loading = PluginRuntime.create(plugin_id, entry)
        .then(runtime => {
            // 失败不记账：下一次进入这个插件的搜索页时重试
            if (runtime === null) loaded.delete(plugin_id);
            return runtime;
        })
        .catch(err => {
            // 起 Worker 就抛了（URL 坏了、CSP 拦了……）：收成"这一次没有插件"，别把拒绝留到调用方
            void warn(`[plugin] load failed: ${plugin_id} ${String(err)}`);
            loaded.delete(plugin_id);
            return null;
        });

    loaded.set(plugin_id, loading);

    return loading;
};

/**
 * 跑一次 `Plugin Search`：装载 → 插件自己的搜索函数
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
        const runtime = await ensure_loaded(plugin_id, entry);
        if (runtime === null) return [];

        return await runtime.search(keyword);
    } catch (err) {
        // Worker 起不来、消息发不出去……都在这里收住
        void warn(`[plugin] search failed: ${plugin_id} ${String(err)}`);
        return [];
    }
};

/**
 * 结果行的一个动作：交回插件自己跑
 *
 * 这条路是单向的（`Plugin::run_action` 是同步的，见 `plugin_impl_js`），所以插件抛错
 * 只在这里记一条日志——Rust 那边已经按"派发过了"处理了。
 */
const on_action_request = async (payload: ActionRequestPayload) => {
    const row = reported.get(payload.plugin_id)?.[payload.row_id];

    if (row === undefined) {
        void warn(`[plugin] action on an unknown row: ${payload.plugin_id} ${payload.row_id}`);
        return;
    }

    const runtime = await loaded.get(payload.plugin_id);
    if (runtime === undefined || runtime === null) {
        void warn(`[plugin] action on a plugin that is not loaded: ${payload.plugin_id}`);
        return;
    }

    runtime.run(row, payload.action_id);
    void info(`[plugin] action dispatched: ${payload.plugin_id} ${payload.action_id}`);
};
