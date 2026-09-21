/** 连续切换的限流间隔（按住方向键的连发、滚轮）：首次立即响应，之后最快 50ms 一次 */
export const SELECT_REPEAT_MS = 50;

/**
 * 预请求的余量：指针落到已加载列表的后 10 位就续下一页。写死 10，不跟随 main_item_n
 *
 * `PREFETCH_MARGIN` 与 `SELECT_REPEAT_MS` 合起来是后端的时间预算：
 * - 假设窗口设定余量为 n （具体见 `Body` 里的计算），即选中行在窗口的倒数第 n 行时向下时，优先尝试移动整个窗口，窗口到底后才移动选中行；
 * - 请求会在选中行上到达最后 10 行时发出，而连续按住下时 `SELECT_REPEAT_MS` 限流到最快 50ms 一次；
 * - 所以后端需要在 `(PREFETCH_MARGIN - n) * SELECT_REPEAT_MS` 内返回，否则高亮行提前下移，等预请求的结果返回会有跳变。
 */
export const PREFETCH_MARGIN = 10;
