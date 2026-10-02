import type { ReactNode } from "react";

/**
 * JS 插件宿主自己的图标
 *
 * 这一份画的是**插件条目**（`js_plugin` 类型）与它唯一的动作（`open_search`）：
 * 插件条目是宿主给每个 `Plugin Package` 画的那一行，不是插件自己注册的类型，
 * 所以图标也由宿主提供。
 *
 * 形状与线宽沿用 launcher 那一套（见 `launcher_icons.tsx`）：尺寸与颜色都由 css 决定。
 */

/* 插件包：一个插座，两根插脚在上面、一根线在下面 */
export const IconJsPlugin = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M6 2.6v3M10 2.6v3"></path>
            <path d="M4 5.6h8v2.6a4 4 0 0 1-8 0z"></path>
            <path d="M8 12.2v1.4"></path>
        </svg>
    );
}

/* 打开插件搜索：放大镜 */
export const IconOpenSearch = (): ReactNode => {
    return (
        <svg className="item-action-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="7" cy="7" r="4.2"></circle>
            <path d="M10.1 10.1 13.4 13.4"></path>
        </svg>
    );
}
