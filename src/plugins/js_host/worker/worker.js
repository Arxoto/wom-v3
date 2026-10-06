// @ts-check

/**
 * @typedef {object} PluginHostApi
 * @property {(spec: unknown) => void} register
 * @property {(spec: unknown) => Promise<unknown>} open_window
 * @property {(command: string, payload: unknown) => Promise<unknown>} request
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
let next_request_id = 1;

/**
 * @typedef {object} PendingRequest
 * @property {(value: unknown) => void} resolve
 * @property {(reason: unknown) => void} reject
 */

/** @type {Map<number, PendingRequest>} */
const pending_requests = new Map();

/**
 * @param {PluginWire.WorkerToHost} message
 */
const post = message => post_to_host(message);

/**
 * @param {string} command
 * @param {unknown} payload
 * @returns {Promise<unknown>}
 */
const request = (command, payload) => new Promise((resolve, reject) => {
    const request_id = next_request_id++;

    pending_requests.set(request_id, { resolve, reject });
    post({ kind: "request", request_id, command, payload });
});

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
    open_window: spec => request("open_window", spec),
    request,
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
        case "response": {
            const pending = pending_requests.get(message.request_id);
            if (pending === undefined) return;

            pending_requests.delete(message.request_id);
            if (message.ok) pending.resolve(message.value);
            else pending.reject(new Error(message.reason));
            return;
        }
    }
});
