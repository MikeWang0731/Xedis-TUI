use crate::ui::theme::ThemePalette;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs},
    Frame,
};

pub struct HelpPopup;

impl HelpPopup {
    pub const TAB_TITLES: &'static [&'static str] = &[
        " [1] 快捷键速查 ",
        " [2] 快捷宏与路由 ",
        " [3] 核心排障指南 ",
        " [4] 安全护栏规范 ",
    ];

    pub fn render(
        f: &mut Frame,
        area: Rect,
        active_tab: usize,
        scroll_offset: usize,
        theme: &ThemePalette,
    ) {
        let popup_w = (area.width.saturating_sub(6)).clamp(60, 100);
        let popup_h = (area.height.saturating_sub(4)).clamp(18, 32);

        let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
        let popup_y = area.y + (area.height.saturating_sub(popup_h)) / 2;

        let modal_area = Rect {
            x: popup_x,
            y: popup_y,
            width: popup_w,
            height: popup_h,
        };

        // 1. Clear modal background
        f.render_widget(Clear, modal_area);

        let modal_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.help_border))
            .title(Line::from(vec![
                Span::styled(" [XEDIS] ", Style::default().bg(theme.help_brand_bg).fg(theme.help_brand_fg).add_modifier(Modifier::BOLD)),
                Span::styled(" 新手秘籍与排障知识库 (Handbook) ", Style::default().fg(theme.text_primary).add_modifier(Modifier::BOLD)),
                Span::styled(" [Tab/1~4 切换分类 · ↑/↓ 滚动 · Esc/F1 退出] ", Style::default().fg(theme.text_muted)),
            ]));

        let inner = modal_block.inner(modal_area);
        f.render_widget(modal_block, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Tabs
                Constraint::Min(8),    // Content
                Constraint::Length(1), // Horizontal separator
                Constraint::Length(1), // Footer info
            ])
            .split(inner);

        // 2. Tabs
        let tabs = Tabs::new(Self::TAB_TITLES.iter().cloned())
            .block(
                Block::default()
                    .borders(Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.help_tab_border)),
            )
            .select(active_tab % Self::TAB_TITLES.len())
            .style(Style::default().fg(theme.help_tab_inactive))
            .highlight_style(Style::default().fg(theme.help_tab_active).add_modifier(Modifier::BOLD));
        f.render_widget(tabs, chunks[0]);

        // 3. Content
        let content_lines = match active_tab % Self::TAB_TITLES.len() {
            0 => Self::get_keybindings_content(chunks[1].width, theme),
            1 => Self::get_macros_content(chunks[1].width, theme),
            2 => Self::get_troubleshoot_content(chunks[1].width, theme),
            _ => Self::get_safety_guard_content(chunks[1].width, theme),
        };

        let total_lines = content_lines.len();
        let visible_h = chunks[1].height as usize;
        let max_scroll = total_lines.saturating_sub(visible_h);
        let effective_scroll = scroll_offset.min(max_scroll);

        let start_idx = effective_scroll;
        let end_idx = (start_idx + visible_h).min(total_lines);

        let visible_items: Vec<ListItem> = if total_lines > 0 && start_idx < total_lines {
            content_lines[start_idx..end_idx]
                .iter()
                .cloned()
                .map(ListItem::new)
                .collect()
        } else {
            Vec::new()
        };

        f.render_widget(List::new(visible_items), chunks[1]);

        // 4. Horizontal Separator
        let sep_str = "─".repeat(chunks[2].width as usize);
        let sep_line = Line::from(Span::styled(sep_str, Style::default().fg(theme.help_tab_border)));
        f.render_widget(Paragraph::new(sep_line), chunks[2]);

        // 5. Footer hint
        let footer_line = Line::from(vec![
            Span::styled(" 导航: ", Style::default().fg(theme.text_muted)),
            Span::styled("[Tab] / [1~4] / [←/→] ", Style::default().fg(theme.help_title_cyan)),
            Span::styled("切换分类 · ", Style::default().fg(theme.text_muted)),
            Span::styled("[↑/↓] / [PageUp/Dn] ", Style::default().fg(theme.help_title_cyan)),
            Span::styled("滚动内容 · ", Style::default().fg(theme.text_muted)),
            Span::styled("[Esc] / [F1] / [q] ", Style::default().fg(theme.help_title_yellow)),
            Span::styled("关闭帮助", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(footer_line), chunks[3]);
    }

    fn get_keybindings_content(_w: u16, theme: &ThemePalette) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" 快捷键速查矩阵 (Keybindings Matrix)", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  • Tab                  ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("在【左侧命令输入流】与【右侧多维监控看板】之间无缝切换操作焦点", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • F5 / Ctrl + B        ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("快速循环切换 4 种分栏布局预设 (平衡 58% | 沉浸 75% | 运维 35% | 极简 100%)", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • Ctrl + [ / Ctrl + ]  ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("以 5% 步长向左/向右自由微调左右分栏比例", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • F2 / F3 / F4         ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("一键直达右侧监控看板 Tab (F2: 监控概览 | F3: 集群拓扑分片 | F4: 慢查与排障建议)", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • 1 / 2 / 3            ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("在右侧看板获得焦点时，按数字键快速切换对应视图", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • /                    ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("唤起常用运维与安全快捷宏菜单 (/theme, /scan, /bigkeys, /slowlog, /interval, /settings 等)", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • @                    ", Style::default().fg(theme.help_title_purple).add_modifier(Modifier::BOLD)),
                Span::styled("唤起集群物理节点选择器 (@node-1, @192.168.1.10:6379, @all 广播等)", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • Up / Down            ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("输入框中浏览历史命令；下拉补全激活时上下选择建议条目；右侧看板上下滚动", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • PageUp / PageDn      ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("快速平滑翻页浏览左侧交互历史记录或右侧集群拓扑树", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • Ctrl + L / /clear    ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("清空当前命令流可视卡片，释放屏幕空间", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • Ctrl + U             ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("快速清除当前输入框缓冲区内容", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • F1 / ?               ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("呼出本新手排障与快捷指南知识库浮层", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • Esc                  ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("关闭弹窗 / 取消命令补全 / 退出当前模态 / 将焦点归还左侧输入框", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • Ctrl + C / Ctrl + Q  ", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled("安全退出 Xedis-TUI 工具", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
        ]
    }

    fn get_macros_content(_w: u16, theme: &ThemePalette) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" 内置快捷指令宏 (Macros) 与集群路由", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  • /theme [dark|light|toggle] ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("无缝切换深色/浅色高对比度主题 (适配白色背景终端)，例: ", Style::default().fg(theme.text_primary)),
                Span::styled("/theme light", Style::default().fg(theme.help_title_green)),
            ]),
            Line::from(vec![
                Span::styled("  • /scan [match] [count]   ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("非阻塞安全游标迭代 (替代极度危险的 KEYS *)，例: ", Style::default().fg(theme.text_primary)),
                Span::styled("/scan user:* 50", Style::default().fg(theme.help_title_cyan)),
            ]),
            Line::from(vec![
                Span::styled("  • /bigkeys [count]        ", Style::default().fg(theme.help_title_green).add_modifier(Modifier::BOLD)),
                Span::styled("一键分析并排行当前数据库中内存占用最大、元素最多的大 Key", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • /slowlog [count]        ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("拉取最新执行慢查询并自动匹配专家级诊断调优建议", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • /interval [ms|s|pause]  ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("动态设置后台 QPS/CPU/内存采样频率，例: ", Style::default().fg(theme.text_primary)),
                Span::styled("/interval 500ms", Style::default().fg(theme.help_title_green)),
                Span::styled(" 或 ", Style::default().fg(theme.text_secondary)),
                Span::styled("/interval pause", Style::default().fg(theme.help_title_red)),
            ]),
            Line::from(vec![
                Span::styled("  • /settings               ", Style::default().fg(theme.help_title_purple).add_modifier(Modifier::BOLD)),
                Span::styled("查看当前直连端点、协议拓扑模式、主题配色、配置文件路径与刷新周期", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • /clients                ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("查看所有外部客户端连接 IP、空闲时长 (Idle) 与最近执行指令", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  • /clear                  ", Style::default().fg(theme.text_muted).add_modifier(Modifier::BOLD)),
                Span::styled("清空当前交互流历史视图 (等同于 Ctrl+L)", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" @ 节点定向路由系统 (Node Picker)", Style::default().fg(theme.help_title_purple).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  • @node-id <command>      ", Style::default().fg(theme.help_title_purple).add_modifier(Modifier::BOLD)),
                Span::styled("绕过 Slot 槽位计算，强制向指定 Master/Replica 节点派发命令，例: ", Style::default().fg(theme.text_primary)),
                Span::styled("@node-1 INFO memory", Style::default().fg(theme.help_title_cyan)),
            ]),
            Line::from(vec![
                Span::styled("  • @all <command>          ", Style::default().fg(theme.cmd_broadcast_fg).add_modifier(Modifier::BOLD)),
                Span::styled("向集群所有分片并发广播执行命令并聚合呈现每个节点的执行结果", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
        ]
    }

    fn get_troubleshoot_content(_w: u16, theme: &ThemePalette) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" Redis 生产运维核心排障实战指引", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" 1. 内存突增与 OOM 异常排查 (Memory Surge & OOM)", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 现象: ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("内存看板 Used 逼近 Maxmemory，客户端写入报错 'OOM command not allowed'.", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 排查: ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("1) 执行 ", Style::default().fg(theme.text_secondary)),
                Span::styled("/bigkeys", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" 排查未设置过期时间的巨型 Hash/Set/List;", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("          2) 执行 ", Style::default().fg(theme.text_secondary)),
                Span::styled("MEMORY USAGE <key>", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" 评估单 Key 字节占用;", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("          3) 观察内存碎片率 (Frag Ratio)，若 > 1.5 可执行 ", Style::default().fg(theme.text_secondary)),
                Span::styled("MEMORY PURGE", Style::default().fg(theme.help_title_green).add_modifier(Modifier::BOLD)),
                Span::styled(" 归还脏页.", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 建议: ", Style::default().fg(theme.help_title_green).add_modifier(Modifier::BOLD)),
                Span::styled("确保配置了合理的淘汰策略 (如 volatile-lru / allkeys-lru)，大对象按业务拆分.", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" 2. CPU 飙高 100% 与慢查询治理 (High CPU & Slow Commands)", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 现象: ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("CPU Sparkline 持续冲顶，Ping 延迟从 0.5ms 骤增至数百毫秒，导致业务连接超时.", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 排查: ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("1) 切换右侧 ", Style::default().fg(theme.text_secondary)),
                Span::styled("[F4] 慢查排障", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" 查看近期慢查询记录;", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("          2) 严禁在生产执行 ", Style::default().fg(theme.text_secondary)),
                Span::styled("KEYS *", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled("、全量 ", Style::default().fg(theme.text_secondary)),
                Span::styled("SMEMBERS", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled(" 或无 LIMIT 的 ", Style::default().fg(theme.text_secondary)),
                Span::styled("HGETALL", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled(";", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("          3) 检查 Lua 脚本中是否存在大范围遍历或死循环.", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 建议: ", Style::default().fg(theme.help_title_green).add_modifier(Modifier::BOLD)),
                Span::styled("全面替换为 ", Style::default().fg(theme.text_primary)),
                Span::styled("/scan", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" 游标分批拉取，将聚合计算移至应用层完成.", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" 3. 连接风暴与阻塞排查 (Connection Spikes & Leaks)", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 现象: ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("活跃连接数持续飙升，达到 maxclients 上限报错 'ERR max number of clients reached'.", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 排查: ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("执行 ", Style::default().fg(theme.text_secondary)),
                Span::styled("/clients", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("，检查是否存在大量 idle 秒数极高但未释放的陈旧连接;", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 建议: ", Style::default().fg(theme.help_title_green).add_modifier(Modifier::BOLD)),
                Span::styled("检查应用端是否正确使用连接池管理；设置 redis.conf timeout 300 自动断开空闲连接.", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" 4. 集群拓扑分片与主从同步异常 (Cluster & Replication Issues)", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 现象: ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("出现 'CLUSTERDOWN' 错误，或者写入时频繁发生 MOVED / ASK 异常重定向.", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 排查: ", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("按 ", Style::default().fg(theme.text_secondary)),
                Span::styled("[F3]", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" 展开集群拓扑看板，检查 0~16383 槽位是否 100% 完整覆盖，从节点同步是否滞后;", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  ▸ 建议: ", Style::default().fg(theme.help_title_green).add_modifier(Modifier::BOLD)),
                Span::styled("若 Master 宕机可由从节点发起 ", Style::default().fg(theme.text_primary)),
                Span::styled("CLUSTER FAILOVER", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled("；使用 redis-cli --cluster check 修复未分配槽位.", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
        ]
    }

    fn get_safety_guard_content(_w: u16, theme: &ThemePalette) -> Vec<Line<'static>> {
        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" Safety Guard 高危命令安全护栏规范", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" 为保障生产环境稳定性，Xedis-TUI 内置多级危险操作拦截机制：", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" [LEVEL 3 - 高危绝对阻断 (Blocking)]", Style::default().bg(theme.guard_l3_badge_bg).fg(theme.guard_l3_badge_fg).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  • FLUSHALL / FLUSHDB  ", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled("彻底清空当前或全部数据库，造成不可逆数据丢失", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  • SHUTDOWN            ", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled("直接终止服务端进程，导致线上服务不可用", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  • KEYS * / pattern    ", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled("O(N) 全库扫描长时间阻塞主线程，引发请求雪崩 (推荐用 /scan)", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  • CONFIG REWRITE      ", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled("改写物理配置文件，可能导致配置漂移或注释丢失", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  • DEBUG SEGFAULT      ", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled("强制进程段错误崩溃退出", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  • DEBUG SLEEP         ", Style::default().fg(theme.help_title_red).add_modifier(Modifier::BOLD)),
                Span::styled("主线程强制休眠挂起", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" [LEVEL 2 - 风险操作告警 (Warning)]", Style::default().bg(theme.guard_l2_badge_bg).fg(theme.guard_l2_badge_fg).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  • SMEMBERS / HGETALL  ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("大集合/哈希全量拉取，易产生大网络包与单次高耗时 (推荐 SSCAN/HSCAN)", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  • REPLICAOF NO ONE    ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("从节点脱离集群，可能引发数据分叉或脑裂 (推荐 CLUSTER FAILOVER)", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  • SWAPDB / MIGRATE    ", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("变更键空间映射或跨节点迁移锁定", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(vec![
                Span::styled("  • BGSAVE / BGREWRITEAOF", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled("手动触发 Fork 导致瞬时 Copy-On-Write 内存翻倍", Style::default().fg(theme.text_secondary)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" 拦截交互操作:", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" 弹出确认浮层后，按 ", Style::default().fg(theme.text_primary)),
                Span::styled("[Enter] / ['y']", Style::default().fg(theme.help_title_yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" 二次确认放行，按 ", Style::default().fg(theme.text_primary)),
                Span::styled("[Esc] / ['n']", Style::default().fg(theme.help_title_cyan).add_modifier(Modifier::BOLD)),
                Span::styled(" 放弃并修改指令。", Style::default().fg(theme.text_primary)),
            ]),
            Line::from(""),
        ]
    }
}
