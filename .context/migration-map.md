# 遗留上下文迁移清单

本清单是迁移闸门。`.context/migration-state.json` 是 version 2 内容级台账；每个遗留文件已按 Markdown 标题或段落切分为稳定内容单元。文件级“已迁移”声明不能通过审计。

## 来源根目录

- `prompt.md` → 逐内容单元分类写入 `AGENTS.md`、`.context/system/`、计划或任务；不得整文件机械改名。
- `.memory` → 逐内容单元分类写入 `AGENTS.md`、`.context/system/`、计划或任务；不得整文件机械改名。
- `.prompt` → 逐内容单元分类写入 `AGENTS.md`、`.context/system/`、计划或任务；不得整文件机械改名。

## 待处置文件

- [ ] `.memory/README.md`
- [ ] `.memory/changelog.md`
- [ ] `.memory/commands.md`
- [ ] `.memory/decisions.md`
- [ ] `.memory/handoff.md`
- [ ] `.memory/project.md`
- [ ] `.memory/status.md`
- [ ] `.memory/tasks.md`
- [ ] `.memory/ui-assets.md`
- [ ] `.prompt/001-phase-0-gpui-bootstrap.md`
- [ ] `.prompt/002-phase-1-gpui-design-system.md`
- [ ] `.prompt/003-phase-2-splash-dashboard.md`
- [ ] `.prompt/004-phase-3-business-pages.md`
- [ ] `.prompt/005-phase-4-core-logic-integration.md`
- [ ] `.prompt/006-phase-5-advanced-features.md`
- [ ] `.prompt/007-phase-6-kernel-integration.md`
- [ ] `.prompt/008-phase-7-kernel-orchestration.md`
- [ ] `.prompt/009-phase-8-real-integration.md`
- [ ] `.prompt/010-phase-9-polish-ux.md`
- [ ] `.prompt/011-phase-10-production-polish.md`
- [ ] `.prompt/012-liora-ui-rebuild-followup.md`
- [ ] `.prompt/README.md`
- [ ] `.prompt/plans/2026-05-02-cleanup-and-gpui-setup.md`
- [ ] `.prompt/plans/2026-05-02-gpui-bootstrap.md`
- [ ] `prompt.md`

## 内容单元处置规则

- `agent_rule` → `AGENTS.md`
- `project_summary`、`business_boundary` → `AGENTS.md` 或 `.context/system/overview.md`
- `system_fact` → `.context/system/overview.md`
- `convention` → `.context/system/conventions.md`
- `risk` → `.context/system/risks.md`
- `plan` → `.context/plans/*.md`
- `task` → `.context/tasks/*.md`
- 只有 `duplicate`、`obsolete`、`disproven`、`sensitive` 可以标记为 `rejected`，且必须记录有证据的理由。

每个 `migrated` 内容单元必须在规范目标中保留完整来源内容，并紧邻唯一来源标记：

```text
<!-- ctx-migration source="<来源路径>" unit="<单元编号>" sha256="<单元哈希>" -->
```

先运行 `python scripts/context_bootstrap.py audit-migration --root <仓库>`；只有覆盖率 100%、分类目标正确、来源标记和哈希全部匹配时，才可运行 `finalize --apply`。原始字节将归档到 `.context/archive/legacy-sources/` 并移除旧入口。
