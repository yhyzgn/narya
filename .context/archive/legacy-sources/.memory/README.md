# .memory 记忆库说明

此目录是 Narya 项目的长期上下文记忆库。任何 AI Agent 在开发前必须读取，开发后必须更新。

## 必读顺序

1. `prompt.md`
2. `.memory/status.md`
3. `.memory/tasks.md`
4. `.memory/decisions.md`
5. `.memory/ui-assets.md`
6. `.memory/handoff.md`
7. `.prompt/README.md`
8. `.prompt/` 中最新阶段 Prompt

## 更新规则

每次开发推进后至少更新：

- `status.md`
- `tasks.md`
- `changelog.md`
- `handoff.md`

不得把临时想法只留在聊天窗口里。凡是会影响后续开发的内容，都要落盘到本目录。
