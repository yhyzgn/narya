# UI 资源与实现映射

## 资源目录

```text
ui/
```

## 一级页面

| 页面 | UI 图 | 建议路由 |
|---|---|---|
| Splash | `ui/splash.png` | 启动页 |
| Dashboard | `ui/dashboard.png` | `/dashboard` |
| 节点 | `ui/nodes.png` | `/nodes` |
| 配置 | `ui/config.png` | `/profiles` 或 `/config` |
| 订阅 | `ui/subscriptions.png` | `/subscriptions` |
| 连接 | `ui/connections.png` | `/connections` |
| 规则 | `ui/rules.png` | `/rules` |
| 日志 | `ui/logs.png` | `/logs` |
| 工具箱 | `ui/tools.png` | `/tools` |
| 设置 | `ui/settings.png` | `/settings` |

## 二级页面重点

- Dashboard：系统代理规则模式、TUN 智能路由
- 节点：测速、节点详情
- 配置：可视化编辑、YAML 编辑
- 订阅：添加方式、URL、文件、剪贴板、二维码、Hiddify、预览、成功
- 连接：连接详情
- 规则：规则编辑器、规则模拟器
- 日志：导出诊断
- 工具箱：DNS、MTR、端口检测、GeoIP、Speed Test、一键诊断
- 设置：外观、网络、IPv6、内核、TUN、DNS、安全、通知、更新、高级

## 视觉实现关键

- AppShell 必须统一。
- SidebarNav 必须与所有图一致。
- 顶部标题/操作栏必须统一。
- 底部状态栏必须统一。
- 使用 Design Tokens 管理颜色、阴影、圆角、字号。
- 图标统一使用 Lucide Icons，不使用 emoji。


## Element Plus 与视觉真源关系

- `ui/` PNG 仍是最高优先级视觉真源，Element Plus 只能作为交互与基础控件实现手段。
- Splash / Dashboard / AppShell 的核心视觉结构必须继续按图自定义实现，不能为了使用组件库牺牲 100% 还原目标。
- 业务页面可使用 Element Plus：Table、Form、Input、Select、Switch、Dialog、Drawer、Tabs、Dropdown、Tooltip、Popover、Message、Notification、Upload、Pagination。
- 使用 Element Plus 前必须覆盖主题变量，并对照 UI 图固定控件高度、字号、间距、圆角和边框透明度。


## UI Specs 交接资料

- `ui/specs/` 是为 Gemini/Codex 生成的文字化实现规格，不内嵌 PNG，不替代 PNG。
- 必读入口：
  - `ui/specs/README.md`
  - `ui/specs/_global-design-system.spec.md`
  - `ui/specs/_page-component-matrix.spec.md`
  - `ui/specs/gemini-handoff.json`
  - `ui/specs/spec-index.json`
- 每张 PNG 都有同名 spec，例如 `ui/dashboard.png` -> `ui/specs/dashboard.spec.md`。
- UI 图变更后必须运行 `python3 scripts/generate-ui-specs.py` 重新生成规格。
