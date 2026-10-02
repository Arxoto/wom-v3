/**
 * JS 插件的执行侧：一个 Worker 跑一个包
 *
 * 这是经典 Worker（`new Worker(url, { type: "classic" })`），所以：
 *
 * - 取插件代码只能用 `importScripts`，不能用 `import`；这个文件因此**一个 import/export
 *   都没有**（连 `import type` 也不行，TypeScript 会给它补 `export {}`），消息类型声明成全局的；
 * - 插件跑在自己的全局里，摸不到宿主文档：这里没有 DOM，也没有 `window`，
 *   宿主接口挂在 `self.__WOM_PLUGIN__` 上。
 *
 * 插件交出来的东西分两半：类型 / 图标 / 文案是纯数据，过线给宿主；`search` 与 `run` 是函数，
 * 不能结构化克隆，留在本文件里。宿主发关键字与行下标过来，这里跑完把结果行发回去。
 */

/** 宿主挂给插件的那一个窄接口：插件能做的事就是注册自己、记日志、报错 */
interface PluginHostApi {
    register(spec: unknown): void;
    log(text: unknown): void;
    fail(text: unknown): void;
}

/** Worker 全局上我们用到的那几样（tsconfig 只有 DOM lib，这里显式取窄面） */
interface PluginWorkerScope {
    importScripts(...urls: string[]): void;
    postMessage(message: PluginWire.WorkerToHost): void;
    addEventListener(type: "message", listener: (event: MessageEvent<PluginWire.HostToWorker>) => void): void;
    __WOM_PLUGIN__: PluginHostApi;
}

const scope = self as unknown as PluginWorkerScope;

// 管道先绑一份：插件跑在同一个全局里，它改 `self.postMessage` / `self.importScripts` 是它的事，
// 但不能把宿主的收发拆掉。监听用 addEventListener 也是同理——插件设自己的 onmessage 不影响这条。
const post_to_host = scope.postMessage.bind(scope);
const import_plugin = typeof scope.importScripts === "function" ? scope.importScripts.bind(scope) : null;

/** 插件调过 `register` 没有：只看调没调，内容合不合法由宿主的注册表判定 */
let registered = false;
let search_of: PluginWire.SearchFn | undefined;
let run_of: PluginWire.RunFn | undefined;

const post = (message: PluginWire.WorkerToHost) => post_to_host(message);

const host_api: PluginHostApi = {
    register: spec => {
        if (typeof spec !== "object" || spec === null) {
            post({ kind: "fail", text: "register spec is not an object" });
            return;
        }

        // 函数不能过结构化克隆：两个函数留在这里，跨线只走数据部分
        const data: Record<string, unknown> = {};
        for (const [key, value] of Object.entries(spec as Record<string, unknown>)) {
            if (key === "search" || key === "run") continue;
            data[key] = value;
        }

        const candidate = spec as { search?: unknown, run?: unknown };
        if (typeof candidate.search === "function") search_of = candidate.search as PluginWire.SearchFn;
        if (typeof candidate.run === "function") run_of = candidate.run as PluginWire.RunFn;

        registered = true;
        post({ kind: "register", spec: data });
    },
    log: text => post({ kind: "log", text: String(text) }),
    fail: text => post({ kind: "fail", text: String(text) }),
};

scope.__WOM_PLUGIN__ = host_api;

/** 宿主请 Worker 执行插件的入口 */
const load = (message: Extract<PluginWire.HostToWorker, { kind: "load" }>) => {
    let ok = false;
    let reason = "";

    if (import_plugin === null) {
        reason = "importScripts is unavailable: the worker is not a classic worker";
    } else {
        try {
            import_plugin(message.entry_url);
            ok = true;
            if (!registered) reason = "script loaded but registered nothing";
        } catch (err) {
            reason = `importScripts failed: ${String(err)}`;
        }
    }

    post({ kind: "loaded", request_id: message.request_id, ok, reason });
};

/**
 * 跑一次插件自己的搜索
 *
 * 无论成败都要回一条 `rows`：宿主那一头在等它，漏一条这一次搜索就永远不落地。
 */
const search = async (message: Extract<PluginWire.HostToWorker, { kind: "search" }>) => {
    if (search_of === undefined) {
        post({ kind: "fail", text: "no search function registered" });
        post({ kind: "rows", request_id: message.request_id, rows: [] });
        return;
    }

    try {
        const rows = normalize_rows(await search_of(message.keyword));
        post({ kind: "rows", request_id: message.request_id, rows });
    } catch (err) {
        post({ kind: "fail", text: `search failed: ${String(err)}` });
        post({ kind: "rows", request_id: message.request_id, rows: [] });
    }
};

/** 结果行的一个动作：交回插件自己跑 */
const run = (message: Extract<PluginWire.HostToWorker, { kind: "run" }>) => {
    if (run_of === undefined) {
        post({ kind: "fail", text: `no action handler registered: ${message.action_id}` });
        return;
    }

    try {
        run_of(message.row, message.action_id);
    } catch (err) {
        post({ kind: "fail", text: `action failed: ${String(err)}` });
    }
};

/**
 * 把插件交出来的东西收成结果行的形状
 *
 * **一行都不丢**：宿主按行在数组里的下标派发动作，这里少一行，后面的行就全部错位。
 * 字段认不出来就是空串 / 空表——那一行画不出图标、点不出动作，但它还在原来的位置上。
 *
 * 放在 Worker 里做的另一个理由：归一化之后只剩字符串，结构化克隆一定过得去；
 * 插件返回函数、类实例这类不能克隆的东西时，坏在归一化这一层比坏在 postMessage 那一层好收。
 */
const normalize_rows = (raw: unknown): PluginWire.Row[] => {
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

scope.addEventListener("message", event => {
    const message = event.data;

    switch (message.kind) {
        case "load":
            load(message);
            return;
        case "search":
            void search(message);
            return;
        case "run":
            run(message);
            return;
    }
});
