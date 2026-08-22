# 任务 001：完成上下文初始化扫描

- 状态：已完成
- 计划：`.context/plans/001_context_bootstrap.md`
- 规模：中
- 依赖：无
- 生产行为变更：无

## 任务目标

完成首次仓库扫描，用经过验证的事实替换脚手架中的未知项。

## 范围

- 检查清单文件、构建文件、运行入口、核心模块、测试和执行命令。
- 在 `.context/system/overview.md`、`conventions.md` 和 `risks.md` 中记录证据。
- 通过迁移清单核对遗留提示和记忆来源。

## 非目标

- 不重构生产代码。
- 不修改依赖、数据库结构、部署文件或外部基础设施。
- 长期内容完成核对和审查前不删除遗留上下文。

## 预期文件

- 修改 `.context/system/overview.md`、`conventions.md` 和 `risks.md`。
- 只为证据和生命周期状态修改本任务及其所属计划。

## 验收标准

- 技术、运行时、模块、构建、测试、运行和覆盖率事实都引用仓库证据。
- 当前计划和任务指针保持有效。
- 人工编写的说明保持不变。
- 未确认项和外部副作用明确记录为风险。

## 验证

- 运行扫描过程中识别出的最小安全检查、构建和测试命令。
- 运行 `python scripts/context_bootstrap.py validate --root <repo>`。
- 记录输出和阻塞项；不得把编译成功描述成业务测试成功。

## 风险与回滚

- 如果扫描或测试会初始化未经授权的共享基础设施，立即停止。
- 回滚时只删除生成的脚手架文件，不修改源码或依赖状态。

## 完成记录

- 状态：已完成。
- 证据：`Cargo.toml`、`crates/narya-*/src`、`README.md`、`git status`。
- 迁移审计：`python /home/neo/.codex/skills/ctx/scripts/context_bootstrap.py audit-migration --root .` 返回 185/185、100%。
- 已知风险：旧实现的 IPC framing、TUN、内核安装和规则编译仍未完成，已登记到 `.context/system/risks.md`。
- 下一项任务：`.context/tasks/002-runtime-foundation.md`。
