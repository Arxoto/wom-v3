# Item Action 映射表与元数据

Type: grilling
Status: open
Blocked by: 02

## Question

`ItemType` → `Item Action` 的具体映射表是什么？四个候选动作——复制进剪贴板、用默认浏览器打开、用默认方式打开、在资源管理器中选中该文件——分别挂到哪些 `ItemType` 上，各自第几个是默认动作？元数据命令（暂名 `fetch_item_actions`）返回什么形状：`ItemType` 的键用字符串还是枚举？每条动作的国际化键怎么起名、放哪一层做兜底（金样本？）？

## Answer

待解决。前置事实见 [四类动作的插件覆盖](02-research-item-action-apis.md)：插件能力决定了哪些动作在某些平台根本不可用，映射表要按这个结论来定。
