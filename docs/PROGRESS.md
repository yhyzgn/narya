# Narya Project Progress & Roadmap

## 项目概览
基于 GPUI 和 Sing-box FFI 核心的高性能代理工具。

## 进度回顾

### Stage 1: 核心架构与 eBPF (Done)
- [x] 项目脚手架搭建 (Workspace, API, Utils)
- [x] eBPF 基础流量追踪逻辑实现
- [x] 通用配置模型定义

### Stage 2: Daemon 守护进程 (Done)
- [x] 进程生命周期管理
- [x] Unix Domain Socket (UDS) IPC 通信实现
- [x] 模拟内核启动与停止

### Stage 3: UI 基础 Shell (Done)
- [x] 基于 GPUI 的侧边栏导航
- [x] 多页面布局 (Dashboard, Proxies, Profiles, Rules, Settings)

### Stage 4: Sing-box 真实内核集成 (Done)
- [x] Go 语言 FFI 桥接代码实现
- [x] Rust 侧 SingBoxFfi 封装
- [x] 配置转换器 (Transformer) 实现，解决 `missing tags` 兼容性问题
- [x] 移除 MockCore，打通真实代理链路

### Stage 5: 节点选择与交互逻辑 (Done)
- [x] 订阅抓取与解析
- [x] 节点延迟测试 (TCP Ping)
- [x] 异步更新逻辑重构（自调度轮询模式，修复 Panic）
- [x] 节点选择联动内核重载

### Stage 6: UI 视觉重构与美学提升 (In Progress)
- [x] 侧边栏样式精修（激活状态、指示条、状态底栏）
- [x] 仪表盘卡片重设计（玻璃拟态、排版优化）
- [x] 节点列表美化（延迟勋章、侧边状态条）
- [x] 修复列表滚动失效问题
- [ ] [TODO] 整体配色方案进阶 (Theme refinement)

## 未来规划

### Stage 7: 系统托盘与全局交互 (In Progress)
- [x] 添加 `ksni` 依赖支持 Linux 托盘 (SNI)
- [x] 实现基础系统托盘图标、菜单与状态显示
- [ ] 系统托盘与 GUI 窗口联动 (Show/Hide)
- [ ] 全局快捷键支持

### Stage 8: 搜索与高级过滤 (In Progress)
- [x] 引入 `fuzzy-matcher` 算法支持
- [x] Rules 页面应用搜索升级为模糊匹配并优化排序
- [x] 搜索栏实时显示匹配结果计数
- [ ] 代理节点关键词过滤

### Stage 9: 日志查看器 (Planned)
- [ ] 实时获取 Sing-box 运行日志
- [ ] UI 侧日志流展示与级别过滤

### Stage 10: eBPF 流量增强 (Planned)
- [ ] 应用级流量统计 (Per-app usage)
- [ ] 自动分流规则同步
