# 代码规范与上下文规则

- 优先遵循仓库现有的分层、命名、错误处理和测试规范。
- 修改导出符号前，必须使用可用的代码智能工具检查全部引用。
- 稳定事实写入 `system/`；阶段顺序写入 `plans/`；边界明确的一项变更写入 `tasks/`。
- 证据与不确定性必须分开记录，禁止把猜测写成系统事实。
- 迁移遗留上下文前必须先分类；保留人工编写的说明和未知文件。
- 所有面向人员的上下文文档必须使用中文；路径、命令和无法翻译的技术标识符除外。

## 当前实现约定

- `narya-core` 只承载可序列化领域模型与跨模块契约；平台副作用放在 daemon/platform 边界。
- `narya-rules` 负责规则语义、匹配优先级和目标选择，不直接执行系统代理或内核进程。
- `narya-kernel` 负责内核标识、版本、安装/升级状态与进程编排契约；具体下载、校验和平台路径由 daemon 实现。
- 所有 IPC 消息必须可 framing；在替换当前裸 JSON read 模型前，不得宣称连接状态可靠。
- 连接状态只有在内核健康探针与代理/TUN 应用成功后才能置为已连接；任何一步失败必须回滚已应用的副作用。
- 修改公共结构或方法前先 `rg` 搜索 workspace 全部引用，再补契约测试和单元测试。

## 验证命令

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

GUI 烟测使用 `timeout 8s cargo run -p narya-app`；超时码 124 仅表示人为截断，必须结合启动输出判断。
