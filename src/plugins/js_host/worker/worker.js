// @ts-check

/**
 * @typedef {object} PluginHostApi
 * @property {(spec: unknown) => void} register
 * @property {(spec: unknown) => void} open_window
 * @property {(text: unknown) => void} log
 * @property {(text: unknown) => void} fail
 */

/**
 * @typedef {object} PluginWorkerScope
 * @property {(...urls: string[]) => void} importScripts
 * @property {(message: PluginWire.WorkerToHost) => void} postMessage
 * @property {(type: "message", listener: (event: MessageEvent<PluginWire.HostToWorker>) => void) => void} addEventListener
 * @property {PluginHostApi} __WOM_PLUGIN__
 */

const scope = /** @type {PluginWorkerScope} */ (/** @type {unknown} */ (self));

const post_to_host = scope.postMessage.bind(scope);
const import_plugin = typeof scope.importScripts === "function" ? scope.importScripts.bind(scope) : null;

let registered = false;
/** @type {PluginWire.SearchFn | undefined} */
let search_of;
/** @type {PluginWire.RunFn | undefined} */
let run_of;

/**
 * @param {PluginWire.WorkerToHost} message
 */
const post = message => post_to_host(message);

/** @type {PluginHostApi} */
const host_api = {
    register: spec => {
        if (typeof spec !== "object" || spec === null) {
            post({ kind: "fail", text: "register spec is not an object" });
            return;
        }

        /** @type {Record<string, unknown>} */
        const data = {};
        for (const [key, value] of Object.entries(/** @type {Record<string, unknown>} */ (spec))) {
            if (key === "search" || key === "run") continue;
            data[key] = value;
        }

        const candidate = /** @type {{ search?: unknown, run?: unknown }} */ (spec);
        if (typeof candidate.search === "function") search_of = /** @type {PluginWire.SearchFn} */ (candidate.search);
        if (typeof candidate.run === "function") run_of = /** @type {PluginWire.RunFn} */ (candidate.run);

        registered = true;
        post({ kind: "register", spec: data });
    },
    open_window: spec => {
        const source = typeof spec === "object" && spec !== null ? /** @type {Record<string, unknown>} */ (spec) : {};

        post({
            kind: "open_window",
            path: typeof source.path === "string" ? source.path : "",
            title: typeof source.title === "string" ? source.title : "",
            width: typeof source.width === "number" ? source.width : 0,
            height: typeof source.height === "number" ? source.height : 0,
        });
    },
    log: text => post({ kind: "log", text: String(text) }),
    fail: text => post({ kind: "fail", text: String(text) }),
};

scope.__WOM_PLUGIN__ = host_api;

/**
 * @param {Extract<PluginWire.HostToWorker, { kind: "load" }>} message
 */
const load = message => {
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
 * @param {Extract<PluginWire.HostToWorker, { kind: "search" }>} message
 */
const search = async message => {
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

/**
 * @param {Extract<PluginWire.HostToWorker, { kind: "run" }>} message
 */
const run = message => {
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
 * @param {unknown} raw
 * @returns {PluginWire.Row[]}
 */
const normalize_rows = raw => {
    if (!Array.isArray(raw)) return [];

    return raw.map(row => {
        const source = /** @type {Record<string, unknown>} */ (typeof row === "object" && row !== null ? row : {});
        const action_ids = Array.isArray(source.action_ids)
            ? source.action_ids.filter(/** @type {(id: unknown) => id is string} */ (id => typeof id === "string"))
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
