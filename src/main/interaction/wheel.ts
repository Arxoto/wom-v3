import type { Intent } from "./keys";

/**
 * 滚轮 → 意图
 *
 * 滚轮的滚动量随平台与设备而变：像素制一格几十到上百像素、行制一格 3 行、触摸板一次事件只有几像素。
 * 所以这里只认方向、不认大小——鼠标一格与触摸板滑一下都算一步，快慢交给与按键连发同一套限流
 * （见 `useMainInteraction` 的 `pace_step`），手感因此不随设备和平台漂。
 */
export const resolve_wheel = (event: WheelEvent, shift: boolean): Intent | null => {
    // 有的平台（按住 Shift 时）把滚轮报成横向：`deltaY` 为 0 而 `deltaX` 有值
    const delta = event.deltaY !== 0 ? event.deltaY : event.deltaX;
    // 两个轴的方向约定一致，正数都是「往后」；没有滚动量（横向键一类的空事件）就什么都不做
    if (delta === 0) return null;

    if (shift) return delta > 0 ? "action_next" : "action_prev";
    return delta > 0 ? "select_next" : "select_prev";
}
