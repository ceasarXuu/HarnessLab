# v1.0.5 Job / Task 状态机重梳理

- 状态：In progress（立项，工程计划见下）
- 创建：2026-08-09
- 范围：把 OrnnLab 的 Job 级与 Task（trial）级状态定义与 Harbor 真实语义对齐，消除"执行状态"与"结果质量"混用，并为"重跑失败任务"立项
- 关联文档：[工程计划](engineering-plan.md)、[PRD](../prd.md)、[技术设计](../technical-design.md)、[工程计划总表](../engineering-plan.md)

## 背景与问题

走查中发现定义冲突：

1. **Job 级"failed"混用两件事**：OrnnLab 在 `n_errored_trials > 0` 时把 job 标记为 failed（`harbor_subprocess._status_from_result_payload`），但 Harbor 的结果模型只有执行时间戳与 trial 统计，一个全部 trial 跑完（含失败/异常 trial）的 job 执行层面是"完成"的。
2. **Trial 级状态自相矛盾**：Trial DTO 的 `status` 在存在 `exception_info` 时显示 `failed`，而 Job 明细把它计为 `errored`。
3. **"恢复"与"重跑失败任务"混为一谈**：`harbor job resume` 只续跑未完成 trial（默认仅 CancelledError 可重跑），完成态 job 的失败任务没有重跑入口。

## 目标状态模型（两轴）

### Job 级

- **执行状态**（生命周期，决定能否恢复）：`queued → running → completed`（全部 trial 有终态结果，执行结束，与得分无关）；`interrupted`（执行中断，部分未完成，可 resume）；`cancelled`（用户取消）；`failed`（仅执行级失败：环境/setup/引擎，未跑完 trial）。
- **结果质量**（派生，非状态）：`total / passed / notPassed / errored` 明细（现有 `job_trial_progress`）。

### Task 级

- `pending（未开始）→ running（执行中）→ passed（得分）| notPassed（不得分）| errored（异常，附原因）| cancelled`
- 运行中残留目录：job running 时显示 `running`，job 中断后显示 `interrupted`。

## 文档入口

| 文档 | 负责内容 |
|---|---|
| [工程计划](engineering-plan.md) | 问题定义、验证门、工作单元、阶段、执行追踪（se-good-plan） |
| 本 README | 立项背景、状态模型摘要、文档入口 |
