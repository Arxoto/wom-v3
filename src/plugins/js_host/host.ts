import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { info, warn } from "@tauri-apps/plugin-log";

import { plugin_open_html_window, plugin_report_search_results, type PluginSearchRow } from "../../core";
import { registry, type PluginView } from "../registry.tsx";

const EVENT_SEARCH_REQUEST = "plugin_search_request";

const EVENT_ACTION_REQUEST = "plugin_action_request";

interface SearchRequestPayload {
    plugin_id: string,
    entry: string,
    keyword: string,
}

interface ActionRequestPayload {
    plugin_id: string,
    row_id: number,
    action_id: string,
}

const reported = new Map<string, PluginSearchRow[]>();

export const install_plugin_host = () => {
    void listen<SearchRequestPayload>(EVENT_SEARCH_REQUEST, event => {
        void on_search_request(event.payload);
    });
    void listen<ActionRequestPayload>(EVENT_ACTION_REQUEST, event => {
        void on_action_request(event.payload);
    });
};

class PluginRuntime {
    private readonly plugin_id: string;
    private readonly worker: Worker;
    private next_request_id = 1;
    private load_request_id: number | null = null;
    private readonly pending = new Map<number, (message: PluginWire.WorkerToHost | null) => void>();

    private constructor(plugin_id: string, worker: Worker) {
        this.plugin_id = plugin_id;
        this.worker = worker;
        this.worker.onmessage = event => this.on_message(event.data as PluginWire.WorkerToHost);
        this.worker.onerror = event => this.on_error(event);
    }

    static async create(plugin_id: string, entry: string): Promise<PluginRuntime | null> {
        const worker = new Worker(new URL("./worker/worker.js", import.meta.url), { type: "classic" });
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
                if (is_registration(message.spec)) {
                    registry.register(message.spec);
                } else {
                    void warn(`[plugin] invalid register spec, dropped: ${this.plugin_id}`);
                }
                return;
            case "request":
                void this.on_request(message);
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

    private async on_request(message: Extract<PluginWire.WorkerToHost, { kind: "request" }>): Promise<void> {
        let reply: HostReply;

        try {
            reply = await handle_request(this.plugin_id, message.command, message.payload);
        } catch (err) {
            reply = { ok: false, value: null, reason: String(err) };
        }

        const response: Extract<PluginWire.HostToWorker, { kind: "response" }> = {
            kind: "response",
            request_id: message.request_id,
            ok: reply.ok,
            value: reply.value,
            reason: reply.reason,
        };

        this.worker.postMessage(response);
    }

    private on_error(event: ErrorEvent): void {
        void warn(`[plugin] worker error: ${this.plugin_id} ${event.message}`);

        if (this.load_request_id !== null) {
            const resolve = this.pending.get(this.load_request_id);
            this.pending.delete(this.load_request_id);
            resolve?.(null);
        }
    }
}

const loaded = new Map<string, Promise<PluginRuntime | null>>();

interface HostReply {
    ok: boolean,
    value: unknown,
    reason: string,
}

const handle_request = async (plugin_id: string, command: unknown, payload: unknown): Promise<HostReply> => {
    if (typeof command !== "string") {
        return { ok: false, value: null, reason: "request command is not a string" };
    }

    switch (command) {
        case "open_window":
            return handle_open_window(plugin_id, payload);
        default:
            return { ok: false, value: null, reason: `unknown request: ${command}` };
    }
};

const handle_open_window = async (plugin_id: string, payload: unknown): Promise<HostReply> => {
    const source = typeof payload === "object" && payload !== null ? payload as Record<string, unknown> : {};

    try {
        await plugin_open_html_window(
            plugin_id,
            typeof source.path === "string" ? source.path : "",
            typeof source.title === "string" ? source.title : "",
            typeof source.width === "number" ? source.width : 0,
            typeof source.height === "number" ? source.height : 0,
        );

        return { ok: true, value: null, reason: "" };
    } catch (err) {
        void warn(`[plugin] open window failed: ${plugin_id} ${String(err)}`);
        return { ok: false, value: null, reason: String(err) };
    }
};

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

const ensure_loaded = (plugin_id: string, entry: string): Promise<PluginRuntime | null> => {
    const cached = loaded.get(plugin_id);
    if (cached !== undefined) return cached;

    const loading = PluginRuntime.create(plugin_id, entry)
        .then(runtime => {
            if (runtime === null) loaded.delete(plugin_id);
            return runtime;
        })
        .catch(err => {
            void warn(`[plugin] load failed: ${plugin_id} ${String(err)}`);
            loaded.delete(plugin_id);
            return null;
        });

    loaded.set(plugin_id, loading);

    return loading;
};

const on_search_request = async (payload: SearchRequestPayload) => {
    const rows = await search_rows(payload);

    reported.set(payload.plugin_id, rows);

    try {
        await plugin_report_search_results(payload.plugin_id, rows);
    } catch (err) {
        void warn(`[plugin] report search results failed: ${String(err)}`);
    }
};

const search_rows = async (payload: SearchRequestPayload): Promise<PluginSearchRow[]> => {
    const { plugin_id, entry, keyword } = payload;

    try {
        const runtime = await ensure_loaded(plugin_id, entry);
        if (runtime === null) return [];

        return await runtime.search(keyword);
    } catch (err) {
        void warn(`[plugin] search failed: ${plugin_id} ${String(err)}`);
        return [];
    }
};

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
