// 探针 Plugin Package 的 JS 入口。
//
// 它把"一个 JS 插件要交出的全部东西"各演一遍：
//   1. 结果行的类型图标、动作图标与中文文案（label_key → 中文）；
//   2. 自己的搜索函数：按关键字返回几行，其中一行**故意没有动作**；
//   3. 结果行动作的处理函数：留一行日志，证明动作真的跑到了插件里。
//
// 装载由宿主的 Worker 用 importScripts 完成，插件跑在 Worker 里，
// 所以这里直接调宿主挂在 Worker 全局上的 self.__WOM_PLUGIN__。
// 宿主接口不在就抛错——那也是要验的一种坏法（脚本自己抛错）。

(() => {
    const host = self.__WOM_PLUGIN__;
    if (!host) throw new Error("probe: self.__WOM_PLUGIN__ is missing");

    // 图标只能是一段字符串：插件跑在 Worker 里，手上没有 React（见 registry.tsx 的 PluginIcon）。
    // 用 data URI 是为了不碰文件系统：插件给不了宿主任意路径，只能给自己画一张。
    // 描边颜色写死是因为 <img> 里拿不到 currentColor。
    const icon = (body) => "data:image/svg+xml;utf8," + encodeURIComponent(
        '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="none" ' +
        'stroke="#8a8a8a" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round">' +
        body + '</svg>'
    );

    host.register({
        id: "probe",

        types: [
            {
                the_type: "probe_result",
                // 类型图标：一个圆加一个点
                icon: icon('<circle cx="8" cy="8" r="5.2"></circle><circle cx="8" cy="8" r="1.4"></circle>'),
                actions: [
                    {
                        id: "greet",
                        label_key: "action.probe.probe_result.greet",
                        // 打招呼：一条短线加一个点，形状上与"记录"分得开
                        icon: icon('<path d="M3.5 8.5h6"></path><circle cx="12" cy="8.5" r="1.6"></circle>'),
                    },
                    {
                        id: "log",
                        label_key: "action.probe.probe_result.log",
                        // 记录：三条横线
                        icon: icon('<path d="M4 5h8M4 8h8M4 11h5"></path>'),
                    },
                ],
            },
        ],

        // 中文只住在前端：插件交上来的就是这几条文案（键是清单推出来的四段式）
        labels: {
            "action.probe.probe_result.greet": "打个招呼",
            "action.probe.probe_result.log": "记一行日志",
        },

        // 插件自己的搜索：结果行的类型名、名字、描述与它自己带的动作列表
        search: (keyword) => [
            {
                the_type: "probe_result",
                name: "关键字：" + (keyword || "（空）"),
                desc: "这一行有两个动作，用左右键切换",
                action_ids: ["greet", "log"],
            },
            {
                the_type: "probe_result",
                name: "没有动作的一行",
                desc: "这一行故意不带动作：结果行可以没有动作",
                action_ids: [],
            },
            {
                the_type: "probe_result",
                name: "第二条有动作的行",
                desc: "只有一个动作，默认就是它",
                action_ids: ["log"],
            },
        ],

        // 结果行的动作：宿主把它送回来，插件自己决定做什么
        run: (row, action_id) => {
            host.log("probe run: action=" + action_id + " row=" + (row ? row.name : "?"));
        },
    });
})();
