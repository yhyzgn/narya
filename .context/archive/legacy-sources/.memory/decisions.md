# 技术决策记录

## ADR-001：全面转向 GPUI + Rust 原生开发

- 决策：放弃 Tauri + Vue + Vite 前端栈，将 V1 UI 技术路线改为 GPUI 原生桌面客户端。
- 原因：实际开发验证中，Tauri/前端组件库与高保真定制设计的冲突严重（如组件畸形、窗口控制不稳定、样式覆盖成本高）。GPUI 作为 Rust 原生框架，可提供极致的性能、GPU 加速渲染，并消除跨语言 IPC 通信的开销。
- 影响：移除 `apps/desktop` 目录，所有 UI 渲染与状态管理迁移至 `crates/narya-app`。

## ADR-002：代理内核以 Sidecar 方式接入

- 决策：sing-box、mihomo、xray-core 均以 sidecar 进程接入。
- 原因：崩溃隔离、升级简单、避免 FFI/ABI 风险。
- V1：优先 sing-box。

## ADR-003：UI 效果图作为绝对视觉真源

- 决策：`ui/` 目录下 PNG 是 UI 实现的唯一真理。由于没有现成组件库可用，GPUI 实现必须根据 `.spec.md` 提供的数据自行封装卡片、开关、进度条等控件，不得降级视觉体验。

## ADR-004：必须维护记忆库与阶段 Prompt

- 决策：使用 `.memory/` 保存长期上下文，使用 `.prompt/` 保存阶段接力提示词。
- 原因：确保任何 AI 工具可无缝接手开发。

## ADR-005：每次推进必须验证、提交、推送

- 决策：每次开发推进后必须编译、测试、运行，通过后提交并推送。
- 例外：若仓库未初始化或远程缺失，必须记录阻塞并请求用户处理，不能伪造推送。

## ADR-006：弃用前端组件库，自研 GPUI UI System

- 决策：不再使用任何现成的 Web 组件库（如 Element Plus）。在 `narya-app` 内建立独立的 UI 基础组件系统，封装 Button、Switch、Input、Select、Chart 等元素，复用 Narya Design Tokens。
- 风险：UI 组件基建成本增加。
- 收益：完全摆脱“组件库样式难以覆盖”的痛点，确保 100% 符合设计稿。

## ADR-007：UI 开发采用 PNG + Spec 双依据交接

- 决策：`ui/` 下 PNG 继续作为最终视觉真源；`ui/specs/` 作为 Gemini/Codex 的文字化实现规格手册。
- 维护：当 `ui/*.png` 变化时，运行 `python3 scripts/generate-ui-specs.py`。

## ADR-008：GPUI API 开发严禁盲猜，必须查阅官方真源

- 决策：在开发 GPUI 相关功能时，严禁通过盲猜 API 或依赖可能过时的知识进行闭门造车。
- 执行：必须实时查阅 [Zed 官方开源仓库](https://github.com/zed-industries/zed)、[GPUI 文档](https://docs.rs/gpui) 以及仓库内的 `examples`。
- 原因：GPUI 处于快速迭代期，API 变动频繁，盲猜会导致严重的编译错误和性能隐患。
