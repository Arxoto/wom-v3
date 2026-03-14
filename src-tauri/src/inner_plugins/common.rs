use std::{fmt::Display, path::PathBuf};

/// todo
/// - Clipboard 剪贴板增强，纯文本复制、预览，不做历史管理，太重了，历史管理和增强有
///   - Windows 原生
///   - Ditto(Windows) https://github.com/sabrogden/Ditto
///   - CopyQ(Win/Mac/Linux) https://github.com/hluk/CopyQ
///   - Maccy(macOS) https://github.com/p0deje/Maccy
///   - FlowLauncher/Raycast 等启动软件集成
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    /// 系统命令
    System,
    /// 命令 可复制并自动打开终端、后台执行
    /// - 更自动化一点 AutoHotkey(Windows) / AppleScript(macOS)
    /// - 更集成一点的方案 enigo （仅控制输入无法识别聚焦的窗口），注意必须 app_handle.run_on_main_thread 主线程执行
    Cmd,
    /// 片段 仅允许复制
    Snippets,
    /// 笔记 MarkDownLite 自定义简化语法，窗口渲染
    /// - 使用 React 组件属性 dangerouslySetInnerHTML 实现注入 html 语法
    /// - 使用 React useEffect 对渲染的内容增加事件监听（如最下面的实现）
    /// - 使用 Tauri convertFileSrc 将本地路径转换（或使用自定义协议，需要自己读取文件并根据后缀添加 Response 头）
    /// - 文件变更通知
    /// - 默认样式限制图片显示
    Note,
    /// 网页 支持使用默认浏览器打开、复制连接
    Web,
    /// 文件/文件夹/应用 支持默认方式打开、在文件夹中选中、复制完整路径 行为一样所以合并了
    File,
    /// 基于路径扫描得到文件
    /// - 可以自建数据库索引
    /// - 配合文件变更通知实时更新索引
    /// - 支持排除规则
    Scan,
}

// todo Item 显示

// import { useMemo } from 'react';
// import { convertFileSrc } from '@tauri-apps/api/core';

// const useSafeHtml = (rawHtml: string) => {
//   return useMemo(() => {
//     // 1. 在内存中创建一个虚拟文档
//     const parser = new DOMParser();
//     const doc = parser.parseFromString(rawHtml, 'text/html');

//     // 2. 精准查找所有 img 标签
//     const imgs = doc.querySelectorAll('img');

//     imgs.forEach(img => {
//       const src = img.getAttribute('src');
//       // 3. 只有当它看起来像本地路径时才转换
//       if (src && !src.startsWith('http') && !src.startsWith('data:') && !src.startsWith('asset:')) {
//         img.setAttribute('src', convertFileSrc(src));
//       }

//       // 顺便可以在这里做一些“非暴力”的预处理
//       img.setAttribute('loading', 'lazy'); // 自动开启延迟加载
//       img.setAttribute('draggable', 'false'); // 禁止拖拽
//     });

//     // 4. 返回处理后的 HTML 字符串
//     return doc.body.innerHTML;
//   }, [rawHtml]);
// };

// const StaticRichText = ({ htmlContent }: { htmlContent: string }) => {
//   const containerRef = useRef<HTMLDivElement>(null);

//   useEffect(() => {
//     if (!containerRef.current) return;

//     // 1. 强制对所有图片绑定事件
//     const images = containerRef.current.querySelectorAll('img');
//     const handleClick = (e: Event) => {
//       const target = e.target as HTMLImageElement;
//       console.log('图片被点击了:', target.src);
//       // 这里可以调用 Tauri 的 API 弹出大图预览
//     };

//     images.forEach(img => {
//       img.addEventListener('click', handleClick);
//       // 顺手解决静态 HTML 的图片加载失败显示问题
//       img.style.cursor = 'pointer';
//     });

//     // 2. 清理函数（防止 React 严格模式下重复绑定）
//     return () => {
//       images.forEach(img => img.removeEventListener('click', handleClick));
//     };
//   }, [htmlContent]); // 当内容更新时重新绑定

//   return (
//     <div
//       ref={containerRef}
//       className="prose max-w-none"
//       dangerouslySetInnerHTML={{ __html: htmlContent }}
//     />
//   );
// };

pub struct ItemTypeParsedFailed;

impl TryFrom<&str> for ItemType {
    type Error = ItemTypeParsedFailed;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "System" => Ok(Self::System),
            "Cmd" => Ok(Self::Cmd),
            "Snippets" => Ok(Self::Snippets),
            "Note" => Ok(Self::Note),
            "Web" => Ok(Self::Web),
            "File" => Ok(Self::File),
            "Scan" => Ok(Self::Scan),
            _ => Err(ItemTypeParsedFailed),
        }
    }
}

impl TryFrom<&String> for ItemType {
    type Error = ItemTypeParsedFailed;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        ItemType::try_from(value.as_str())
    }
}

impl From<&ItemType> for &str {
    fn from(value: &ItemType) -> Self {
        match value {
            ItemType::System => "System",
            ItemType::Cmd => "Cmd",
            ItemType::Snippets => "Snippets",
            ItemType::Note => "Note",
            ItemType::Web => "Web",
            ItemType::File => "File",
            ItemType::Scan => "Scan",
        }
    }
}

impl Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = Into::<&str>::into(self);
        write!(f, "{}", type_name)
    }
}

#[derive(Debug, Clone)]
pub enum ItemDesc {
    Str(String),
    Path(PathBuf),
}

impl From<String> for ItemDesc {
    fn from(value: String) -> Self {
        Self::Str(value)
    }
}

impl From<&str> for ItemDesc {
    fn from(value: &str) -> Self {
        Self::Str(value.to_string())
    }
}

impl From<PathBuf> for ItemDesc {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}