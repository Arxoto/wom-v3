declare namespace PluginWire {
    type Row = import("../../core").PluginSearchRow;

    type SearchFn = (keyword: string) => unknown;

    type RunFn = (row: Row | undefined, action_id: string) => void;

    type HostToWorker =
        | { kind: "load", request_id: number, entry_url: string }
        | { kind: "search", request_id: number, keyword: string }
        | { kind: "run", row: Row, action_id: string }
        | { kind: "response", request_id: number, ok: boolean, value: unknown, reason: string };

    type WorkerToHost =
        | { kind: "register", spec: unknown }
        | { kind: "loaded", request_id: number, ok: boolean, reason: string }
        | { kind: "rows", request_id: number, rows: Row[] }
        | { kind: "request", request_id: number, command: string, payload: unknown }
        | { kind: "log", text: string }
        | { kind: "fail", text: string };
}
