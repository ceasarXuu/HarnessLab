# BUG-04: SSE 事件流不工作

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: experiments API, event_service
- Related Links: [README](README.md), [BUG-07](07-event-mirror-quadratic-read-write.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 6（SSE，依赖事件系统稳定）

## 问题描述

`/api/experiments/{experiment_id}/events/stream` 端点声称提供 Server-Sent Events
实时流，但实现上只发送已有事件后立即关闭连接，不轮询新事件。前端无法通过 SSE
实时获取执行进度。

## 证据

文件: `ornnlab/api/experiments.py` 第 100-106 行

```python
@router.get("/{experiment_id}/events/stream")
async def event_stream(experiment_id: str, request: Request, after: int = 0) -> StreamingResponse:
    service = EventService(request.app.state.settings)

    async def stream():
        for event in service.list_after(experiment_id, after):
            yield f"id: {event.id}\nevent: {event.event_type}\ndata: {event.model_dump_json()}\n\n"
        await asyncio.sleep(0.01)  # 然后直接关闭连接

    return StreamingResponse(stream(), media_type="text/event-stream")
```

问题分析：
1. `list_after` 是一次性同步查询，只返回调用时刻已存在的事件
2. `asyncio.sleep(0.01)` 后 generator 结束，连接关闭
3. 没有循环轮询机制，不等待后续产生的新事件
4. 前端如果订阅此流，会在收到已有事件后立即断开

## 修复方案

实现带客户端断开检测的 SSE 轮询循环：

```python
@router.get("/{experiment_id}/events/stream")
async def event_stream(experiment_id: str, request: Request, after: int = 0) -> StreamingResponse:
    settings = request.app.state.settings

    async def stream():
        cursor = after
        while True:
            # 检测客户端是否已断开
            if await request.is_disconnected():
                break

            # 查询新事件
            service = EventService(settings)
            events = service.list_after(experiment_id, cursor)
            for event in events:
                cursor = event.id
                yield f"id: {event.id}\nevent: {event.event_type}\ndata: {event.model_dump_json()}\n\n"

            # 检查 experiment 是否已终止
            exp_service = ExperimentService(settings)
            try:
                state = exp_service.get(experiment_id)
                if state["experiment"]["status"] in {"completed", "failed", "cancelled", "interrupted"}:
                    # 发送剩余事件后关闭
                    remaining = service.list_after(experiment_id, cursor)
                    for event in remaining:
                        cursor = event.id
                        yield f"id: {event.id}\nevent: {event.event_type}\ndata: {event.model_dump_json()}\n\n"
                    yield f"event: stream.end\ndata: {{\"status\": \"{state['experiment']['status']}\"}}\n\n"
                    break
            except KeyError:
                break

            await asyncio.sleep(0.5)  # 轮询间隔

    return StreamingResponse(stream(), media_type="text/event-stream")
```

## 性能影响

每 0.5 秒创建 `EventService` + `ExperimentService` + 2 次 DB 查询。长连接下
开销：

| 连接时长 | DB 查询次数 | 对象创建次数 |
|----------|-------------|--------------|
| 10 秒    | 40          | 40 组        |
| 1 分钟   | 240         | 240 组       |

对于本地单用户 WebUI 场景，此开销可接受。如果未来需要优化，可引入
asyncio.Queue + 事件发布订阅模式，避免轮询。

## 验收标准

- [x] SSE 流持续发送新事件直到 experiment 终止
- [x] 客户端断开连接后服务器停止轮询（通过 `request.is_disconnected()` 检测）
- [x] experiment 终止后发送 `stream.end` 事件并关闭连接
- [x] experiment 不存在时优雅关闭连接
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 实时事件 | 创建 experiment + enqueue，订阅 SSE，等待 completed | 收到 created → queued → completed 事件流 |
| 正确性验证 | 客户端断开 | 订阅 SSE 后立即断开，检查服务器日志 | 服务器停止轮询，无无限循环 |
| 正确性验证 | experiment 终止后关闭 | 等待 experiment completed 后检查 SSE | 收到 stream.end 事件，连接关闭 |
| 回归测试 | 现有事件 API | GET /events（非流式） | 返回正确的事件列表 |

## 回滚策略

单次 commit，`git revert` 即可。API 接口不变，仅流式行为改进。
