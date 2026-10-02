import { useEffect } from "react";

import { Box, Static, Elastic, DividerTop, DividerBottom } from "./main/Layout";
import Head from "./main/Head";
import Body from "./main/Body";
import Tail from "./main/Tail";
import { setup_page_main } from "./core";
import { useMainInteraction } from "./main/interaction/useMainInteraction";
import { registry } from "./plugins/registry.tsx";
import { action_index_of, active_items, active_selection, current_item } from "./main/interaction/reducer";

import "./core.css";

/** 空输入时的占位文案 */
const GHOST_PLACEHOLDER = "输入关键字开始搜索，空格分割参数";

const App = () => {
  useEffect(setup_page_main, []);

  const { state, item_n, input_ref } = useMainInteraction();

  /** 插件搜索页是否打开：它是一份与主列表并列的列表（见 reducer 的 PluginSearchState） */
  const search_open = state.plugin_search !== null;

  // 渲染读的是"当前生效"那一层：搜索页打开时是它，否则是主列表
  const item_list = active_items(state);
  const selection = active_selection(state);
  const action_indices = state.plugin_search?.action_indices ?? state.action_indices;

  /** 空态：主列表还没有结论（没输入过、输入为空，或这一次查询还没回来） */
  const empty = state.conclusion === null;

  const selected_item = current_item(item_list, selection);
  const selected_action_index = selected_item ? action_index_of(state, selected_item.item_index) : 0;

  return (
    <Box>
      <Static>
        <Head
          value={state.input}
          ghost={state.input === "" ? GHOST_PLACEHOLDER : ""}
          input_ref={input_ref}
          read_only={state.preview_open || search_open}
          empty={empty}
          total={state.plugin_search?.total ?? state.conclusion?.total ?? 0}
          selection={selection}>
        </Head>
      </Static>
      <DividerTop></DividerTop>
      <Elastic>
        <Body
          item_list={item_list}
          selection={selection}
          item_n={item_n}
          show_preview={state.preview_open}
          action_indices={action_indices}
          empty={empty}
          search_open={search_open}>
        </Body>
      </Elastic>
      <DividerBottom></DividerBottom>
      <Static>
        <Tail
          preview_open={state.preview_open}
          search_open={search_open}
          action_desc={registry.current_action_label(selected_item, selected_action_index)}>
        </Tail>
      </Static>
    </Box>
  );
}

export default App;

/*
todo list
- 列表项高亮时左侧一个 2-4px 的 Accent Color 条、可延迟动画，（右侧显示常用图标，悬停才显示）
  - 动画用 absolute 定位的元素去实现
- 高亮优化：选中时增加 .is-selected css 类效果，复杂实现使用 Data Attributes
  - HTML <div className="list-item" data-selected={isSelected} data-action-type={item.type} />
  - CSS .list-item[data-selected="true"][data-action-type="copy"] { }
- 使用 React.memo 确保只有选中的行和刚刚失去选中的行发生变化
- （可选）高频触发、复杂的位移或缩放动画，使用 css will-change: background-color; 避免滥用，一般 transform 和 opacity 是值得使用的
- 显示匹配分割线（国际化）：完全匹配、前缀匹配、关键词匹配、不完全匹配，匹配模式名称标签显示在右侧，不占空间、不改变布局（位移）
- 翻页交互：一行一行下翻，并使用边距预览、可见区往下 60% 处触发滚动
  - 下滑操作有限流 100-120 ms
  - 注意 Accent Color 动画拉长回缩，在持续下滑的时候表现出弹力
  - 注意 React 的 key 不变防止销毁 DOM
  - 下翻时给予 list 一个向上的 transform&opacity transition
- Cmd 可自带 AutoHotkey(Windows) / AppleScript(macOS) 脚本实现自动化
  - 可选（较重）：集成 enigo ，注意必须 app_handle.run_on_main_thread 主线程执行
- System 系统命令：关机、重启、睡眠、休眠、锁定、注销、关闭屏幕
- Note MarkDownLite 自定义简化语法，窗口渲染
  - 使用 React 组件属性 dangerouslySetInnerHTML 实现注入 html 语法
  - 使用 React useEffect 对渲染的内容增加事件监听（如最下面的实现）
  - 使用 Tauri convertFileSrc 将本地路径转换（或使用自定义协议，需要自己读取文件并根据后缀添加 Response 头）
  - 文件变更通知
  - 默认样式限制图片显示
- Snippets 片段，仅允许复制
*/

// =================================
// todo Item 实现
// =================================

// 下翻 list 过渡动画，使用 rAF 绕过 React 渲染任务队列、不写明 will-change 让其自动优化
// /* 基础槽位 */
// .item-slot {
//     transform: translateY(0);
//     opacity: 1;
//     /* 回弹阶段的动画：缓动函数决定了“弹力感” */
//     transition: transform 0.2s cubic-bezier(0.175, 0.885, 0.32, 1.275),
//                 opacity 0.2s ease;
// }
// /* 瞬间触发态 */
// .item-slot.is-bumping {
//     /* 关键：关闭过渡，让位移瞬间发生 */
//     transition: none !important;
//     transform: translateY(2px);
//     opacity: 0.7;
// }

// const ListItemSlot = ({ data, isActive }) => {
//     const domRef = useRef<HTMLDivElement>(null);
//     const isFirstRender = useRef(true);

//     useLayoutEffect(() => {
//         if (isFirstRender.current) {
//             isFirstRender.current = false;
//             return;
//         }

//         const el = domRef.current;
//         if (!el) return;

//         // 第一帧：瞬间加上类名，让元素跳到 2px 位置，由于在 useLayoutEffect 中，这一步对用户是不可见的（发生在 Paint 之前）
//         el.classList.add('is-bumping');
//         // 第二帧：移除类名，CSS 的 transition 此时接管，让它弹回 0
//         const rafId = requestAnimationFrame(() => {
//             el.classList.remove('is-bumping');
//         });


//         // 强制重置，防止 React 复用槽位时带着旧的样式
//         return () => {
//             cancelAnimationFrame(rafId);
//             if (el) {
//                 el.classList.remove('is-bumping');
//             }
//         };
//     }, [data?.id]); // 仅在数据更新时触发

//     return (
//         <div ref={domRef} className={`item-slot ${isActive ? 'active' : ''}`}>
//             {/* 内容... */}
//         </div>
//     );
// };

// =================================
// todo Note 实现
// =================================

// import { useMemo } from 'react';
// import { convertFileSrc } from '@tauri-apps/api/core';

// const useSafeHtml = (rawHtml: string) => {
//     return useMemo(() => {
//         // 1. 在内存中创建一个虚拟文档
//         const parser = new DOMParser();
//         const doc = parser.parseFromString(rawHtml, 'text/html');

//         // 2. 精准查找所有 img 标签
//         const imgs = doc.querySelectorAll('img');

//         imgs.forEach(img => {
//             const src = img.getAttribute('src');
//             // 3. 只有当它看起来像本地路径时才转换
//             if (src && !src.startsWith('http') && !src.startsWith('data:') && !src.startsWith('asset:')) {
//                 img.setAttribute('src', convertFileSrc(src));
//             }

//             // 顺便可以在这里做一些“非暴力”的预处理
//             img.setAttribute('loading', 'lazy'); // 自动开启延迟加载
//             img.setAttribute('draggable', 'false'); // 禁止拖拽
//         });

//         // 4. 返回处理后的 HTML 字符串
//         return doc.body.innerHTML;
//     }, [rawHtml]);
// };

// const StaticRichText = ({ htmlContent }: { htmlContent: string }) => {
//     const containerRef = useRef<HTMLDivElement>(null);

//     useEffect(() => {
//         if (!containerRef.current) return;

//         // 1. 强制对所有图片绑定事件
//         const images = containerRef.current.querySelectorAll('img');
//         const handleClick = (e: Event) => {
//             const target = e.target as HTMLImageElement;
//             console.log('图片被点击了:', target.src);
//             // 这里可以调用 Tauri 的 API 弹出大图预览
//         };

//         images.forEach(img => {
//             img.addEventListener('click', handleClick);
//             // 顺手解决静态 HTML 的图片加载失败显示问题
//             img.style.cursor = 'pointer';
//         });

//         // 2. 清理函数（防止 React 严格模式下重复绑定）
//         return () => {
//             images.forEach(img => img.removeEventListener('click', handleClick));
//         };
//     }, [htmlContent]); // 当内容更新时重新绑定

//     return (
//         <div
//             ref={containerRef}
//             className="prose max-w-none"
//             dangerouslySetInnerHTML={{ __html: htmlContent }}
//         />
//     );
// };

