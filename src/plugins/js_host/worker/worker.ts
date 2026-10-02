interface PluginHostApi {
    register(spec: unknown): void;
    log(text: unknown): void;
    fail(text: unknown): void;
}

interface PluginWorkerScope {
    importScripts(...urls: string[]): void;
    postMessage(message: PluginWire.WorkerToHost): void;
    addEventListener(type: "message", listener: (event: MessageEvent<PluginWire.HostToWorker>) => void): void;
    __WOM_PLUGIN__: PluginHostApi;
}

const scope = self as unknown as PluginWorkerScope;

const post_to_host = scope.postMessage.bind(scope);
const import_plugin = typeof scope.importScripts === "function" ? scope.importScripts.bind(scope) : null;

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
