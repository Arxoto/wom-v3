import type { ReactNode } from "react";

/**
 * launcher 插件的条目图标
 *
 * 每个类型配一张纯线条图标，形状要一眼能认出类型。整张图标只用描边，线宽统一 1.2。
 * 图标是行内的一部分，所以尺寸与颜色都由 Item.css 决定（跟着 --item-h 缩放、currentColor 跟随整行状态）。
 *
 * 图标由 launcher 自己提供、注册进前端注册表（见 `launcher.tsx`）：类型名是插件定义的，
 * 「这个类型长什么样」自然也是插件的事。
 *
 * snip / note 两张图标在接入这一轮**删掉了**：那两个类型还没有插件，留着就是没有调用方的
 * 死图标；等它们各自成为插件（spec §2.1 理由 1）时，从 git 历史里取回来即可。
 */

/* 系统命令：齿轮，外圈八个齿、内圈一个孔 */
export const IconSys = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="8" cy="8" r="3.3"></circle>
            <circle cx="8" cy="8" r="1.3"></circle>
            <path d="M12.2 8h1.3M8 12.2v1.3M3.8 8H2.5M8 3.8V2.5M11 11l.9.9M5 11l-.9.9M5 5 4.1 4.1M11 5l.9-.9"></path>
        </svg>
    );
}

/* 命令：终端窗口，框内是提示符 >_ */
export const IconCmd = (): ReactNode => {
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
export const IconWeb = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="8" cy="8" r="5.6"></circle>
            <path d="M2.4 8h11.2"></path>
            <ellipse cx="8" cy="8" rx="2.6" ry="5.6"></ellipse>
        </svg>
    );
}

/* 扫描：一列结果（三条线）后面跟一把放大镜 */
export const IconScan = (): ReactNode => {
    return (
        <svg className="item-icon" viewBox="0 0 16 16" aria-hidden="true"
            fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M2.4 4.2h5.8M2.4 7.4h5.8M2.4 10.6h3.4"></path>
            <circle cx="11.4" cy="10.2" r="2.1"></circle>
            <path d="M12.9 11.7 14.3 13.1"></path>
        </svg>
    );
}
