/**
 * 宿主与插件 Worker 之间的消息形状
 *
 * 为什么声明在全局而不是写成一个模块：Worker 那一半是**经典 Worker**，源码里一个
 * import/export 都不能有——TypeScript 会给「只有类型引用」的模块补一句 `export {}`，
 * 经典 Worker 碰到它直接语法报错（Vite dev 下实测如此）。所以这些名字放在这里，
 * 两边都不用 import。
 *
 * `Row` 用 `import()` 类型查询指回 `core.tsx` 的 `PluginSearchRow`：类型查询不是
 * import 语句，不会把这份声明变成模块，形状也就只有一份。
 */
declare namespace PluginWire {
    type Row = import("../core").PluginSearchRow;

    /** 插件自己交上来的搜索函数：给关键字，还它要画的那几行 */
    type SearchFn = (keyword: string) => unknown;

    /** 插件自己交上来的结果行动作处理函数 */
    type RunFn = (row: Row | undefined, action_id: string) => void;

    /** 宿主 → Worker */
    type HostToWorker =
        | { kind: "load", request_id: number, entry_url: string }
        | { kind: "search", request_id: number, keyword: string }
        | { kind: "run", row: Row, action_id: string };

    /** Worker → 宿主 */
    type WorkerToHost =
        /** 插件注册的内容：`search` / `run` 两个函数留在 Worker 里，这一份是可克隆的数据部分 */
        | { kind: "register", spec: unknown }
        /** 入口执行完了：`ok` 只表示脚本没抛错，注册成没成由宿主的注册表说了算 */
        | { kind: "loaded", request_id: number, ok: boolean, reason: string }
        | { kind: "rows", request_id: number, rows: Row[] }
        | { kind: "log", text: string }
        | { kind: "fail", text: string };
}
