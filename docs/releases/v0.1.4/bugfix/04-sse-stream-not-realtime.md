# BUG-04: SSE 事件流不工作

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: experiments API, event_service
- Related Links: [README](README.md), [BUG-07](07-event-mirror-quadratic-read-write.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 6（SSE，依赖事件系统稳定）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

`/api/experiments/{experiment_id}/events/stream` 声称提供 Server-Sent Events 实时流，但当前实现只发送调用时已有事件，然后结束 generator，连接立即关闭。前端无法通过 SSE 实时获取执行进度。

此外，当前事件查询只按 `aggregate_id = experiment_id` 返回 experiment 级事件，而 run 执行中的核心事件（如 `harbor.job.running`、`harbor.job.completed`）写在 run aggregate 上。即使 SSE 保持连接，如果只查 experiment aggregate，也会漏掉主要进度事件。

## 证据

文件: `ornnlab/api/experiments.py`

```python
@router.get("/{experiment_id}/events/stream")
async def event_stream(experiment_id: str, request: Request, after: int = 0) -> StreamingResponse:
    service = EventService(request.app.state.settings)

    async def stream():
        for event in service.list_after(experiment_id, after):
            yield f"id: {event.id}\nevent: {event.event_type}\ndata: {event.model_dump_json()}\n\n"
        await asyncio.sleep(0.01)  # generator 随后结束，连接关闭

    return StreamingResponse(stream(), media_type="text/event-stream")
```

问题点：

1. `list_after` 是一次性查询，只返回调用时已有事件。
2. `asyncio.sleep(0.01)` 后 generator 结束，SSE 连接关闭。
3. 没有轮询、订阅或客户端断开检测。
4. 只查询 experiment aggregate，会漏掉 run aggregate 的执行事件。
5. 通过 experiment terminal 状态关闭流时，可能与 terminal event append 顺序产生竞态。

## 修复方案

### 1. 增加 experiment event 查询辅助方法

`EventService` 建议新增按多个 aggregate id 查询的方法，避免 API 层手写复杂 SQL：

```python
def list_after_many(self, aggregate_ids: list[str], after: int = 0) -> list[EventRecord]:
    if not aggregate_ids:
        return []
    placeholders = ",".join("?" for _ in aggregate_ids)
    with sqlite.connect(self.settings) as conn:
        rows = sqlite.rows(
            conn,
            f"SELECT * FROM experiment_events "
            f"WHERE aggregate_id IN ({placeholders}) AND id > ? ORDER BY id",
            (*aggregate_ids, after),
        )
    return [self._record(row) for row in rows]
```

`_record(row)` 可抽取自现有 `list_after`，避免重复构造 `EventRecord`。

### 2. SSE 流同时包含 experiment 与 run 事件

```python
@router.get("/{experiment_id}/events/stream")
async def event_stream(experiment_id: str, request: Request, after: int = 0) -> StreamingResponse:
    settings = request.app.state.settings

    async def stream():
        cursor = after
        while True:
            if await request.is_disconnected():
                break

            exp_service = ExperimentService(settings)
            try:
                state = exp_service.get(experiment_id)
            except KeyError:
                yield 'event: stream.error\ndata: {"detail":"experiment not found"}\n\n'
                break

            aggregate_ids = [experiment_id, *[run["id"] for run in state["runs"]]]
            service = EventService(settings)
            events = service.list_after_many(aggregate_ids, cursor)
            for event in events:
                cursor = event.id
                yield format_sse(event)

            if _experiment_terminal(state):
                # 给 finalize_experiment_if_terminal 的 event append 留一个短窗口，避免先读到 terminal status 后漏事件。
                await asyncio.sleep(0.1)
                service = EventService(settings)
                remaining = service.list_after_many(aggregate_ids, cursor)
                for event in remaining:
                    cursor = event.id
                    yield format_sse(event)
                yield f"event: stream.end\ndata: {{\"status\": \"{state['experiment']['status']}\"}}\n\n"
                break

            await asyncio.sleep(0.5)

    return StreamingResponse(stream(), media_type="text/event-stream")
```

### 3. SSE 格式工具函数

```python
def format_sse(event: EventRecord) -> str:
    return (
        f"id: {event.id}\n"
        f"event: {event.event_type}\n"
        f"data: {event.model_dump_json()}\n\n"
    )
```

### 4. 终止条件

```python
TERMINAL_EXPERIMENT_STATUSES = {
    "completed",
    "failed",
    "partially_failed",
    "cancelled",
    "interrupted",
}

def _experiment_terminal(state: dict) -> bool:
    return state["experiment"]["status"] in TERMINAL_EXPERIMENT_STATUSES
```

注意：当前状态派生存在 BUG-06 的重复逻辑，SSE 修复应在 BUG-06 后落地。

## 性能影响

每 0.5 秒轮询一次 DB。对于本地单用户 WebUI 可接受。若未来同时存在多个长连接或大量事件，应升级为 event bus / `asyncio.Queue` / pub-sub 模型。

在本阶段中，DB 仍是 source of truth；JSONL mirror 只是诊断副本，SSE 不应读取 mirror 文件。

## Acceptance Criteria（目标，未完成）

- [ ] SSE 流在 experiment 非 terminal 时保持连接，并持续发送新增事件。
- [ ] SSE 同时包含 experiment aggregate 和该 experiment 下所有 run aggregate 的事件。
- [ ] 客户端断开连接后服务器停止轮询。
- [ ] experiment terminal 后发送剩余事件，再发送 `stream.end` 并关闭连接。
- [ ] experiment 不存在时返回 `stream.error` 或等价错误语义，不进入无限循环。
- [ ] 现有 `/events` 非流式 API 保持兼容。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 实时事件 | 创建 experiment + enqueue，订阅 SSE | 收到 queued/running/completed 等事件 |
| 正确性验证 | run 级事件 | 订阅 experiment SSE 后执行 run | 收到 `harbor.job.running` 和 `harbor.job.completed` |
| 正确性验证 | 客户端断开 | 订阅 SSE 后断开 | server 轮询退出，无后台泄漏 |
| 终止竞态 | terminal 状态与 terminal event 顺序 | 模拟 finalize 先更新状态再 append event | 关闭前不漏最后事件 |
| 回归测试 | GET /events | 非流式事件接口 | 返回格式不变 |

## 回滚策略

单次代码 commit 可直接 `git revert`。API 路径不变，仅修正流式行为。
