import type { ReactNode } from "react";

import type { ItemType } from "../../core";

/**
 * Item Type 的图标
 *
 * 每种 ItemType 配一张纯线条图标，形状要一眼能认出类型。整张图标只用描边，线宽统一 1.2。
 * 图标是行内的一部分，所以尺寸与颜色都由 Item.css 决定（跟着 --item-h 缩放、currentColor 跟随整行状态）。
 */

/* 片段：一对尖括号夹一道斜杠，即 </>
   斜杠与左右括号各留 1 单位的空隙，避免描边叠在一起 */
const IconSnip = () => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M5.8 3.8 2.6 8l3.2 4.2"></path>
            <path d="M9.4 3.6 6.8 12.4"></path>
            <path d="M10.4 3.8 13.6 8l-3.2 4.2"></path>
        </svg>
    );
}

/* 系统命令：齿轮，外圈八个齿、内圈一个孔 */
const IconSys = () => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="8" cy="8" r="3.3"></circle>
            <circle cx="8" cy="8" r="1.3"></circle>
            <path d="M12.2 8h1.3M8 12.2v1.3M3.8 8H2.5M8 3.8V2.5M11 11l.9.9M5 11l-.9.9M5 5 4.1 4.1M11 5l.9-.9"></path>
        </svg>
    );
}

/* 笔记：折角文档加两条正文线 */
const IconNote = () => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M4.6 2.6h4.6l2.4 2.4v8.4H4.6z"></path>
            <path d="M9.2 2.6V5h2.4"></path>
            <path d="M6.6 8.4h3.4M6.6 10.8h3.4"></path>
        </svg>
    );
}

/* 命令：终端窗口，框内是提示符 >_ */
const IconCmd = () => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <rect x="2.4" y="3.4" width="11.2" height="9.2" rx="1.6"></rect>
            <path d="M5.2 6.4 7.1 8.3 5.2 10.2"></path>
            <path d="M8.6 10.4h2.6"></path>
        </svg>
    );
}

/* 网页：地球，一条纬线加两条经线（椭圆的两侧就是两条经线） */
const IconWeb = () => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="8" cy="8" r="5.6"></circle>
            <path d="M2.4 8h11.2"></path>
            <ellipse cx="8" cy="8" rx="2.6" ry="5.6"></ellipse>
        </svg>
    );
}

/* 文件：文件夹 */
const IconFile = () => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2 12.3V4.4a1.2 1.2 0 0 1 1.2-1.2h2.7l1.6 1.9h5.3a1.2 1.2 0 0 1 1.2 1.2v5.9a1.2 1.2 0 0 1-1.2 1.2H3.2a1.2 1.2 0 0 1-1.2-1.2z"></path>
        </svg>
    );
}

/* 扫描：一列结果（三条线）后面跟一把放大镜 */
const IconScan = () => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2.4 4.2h5.8M2.4 7.4h5.8M2.4 10.6h3.4"></path>
            <circle cx="11.4" cy="10.2" r="2.1"></circle>
            <path d="M12.9 11.7 14.3 13.1"></path>
        </svg>
    );
}

/** Item 图标表：按 `ItemType` 查 */
export const ITEM_ICONS: Record<ItemType, ReactNode> = {
    snip: <IconSnip></IconSnip>,
    sys: <IconSys></IconSys>,
    note: <IconNote></IconNote>,
    cmd: <IconCmd></IconCmd>,
    web: <IconWeb></IconWeb>,
    file: <IconFile></IconFile>,
    scan: <IconScan></IconScan>,
};
