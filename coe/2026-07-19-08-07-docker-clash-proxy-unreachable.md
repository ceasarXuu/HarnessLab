# Docker 任务容器无法访问 Clash 代理

## Problem

- P-001：宿主机可通过 Clash 访问 Claude、npm 等外部服务，但 Harbor Job 的 trial 容器下载 Claude Code 时连接 `downloads.claude.ai:443` 超时，导致 Agent 非零退出。

## Hypothesis

- H-001（confirmed，all_of）：trial 容器没有获得可用的代理环境变量；同时宿主 Clash 仅监听 `127.0.0.1:7890`，即使原样继承该地址，在容器中也只会指向容器自身。
- H-002（falsified）：Clash 规则或目标站点自身异常，经宿主 Clash 显式代理访问也会失败。

## Evidence

- E-001（supports H-001）：最近两个已完成 trial 均在执行 Claude Code bootstrap 时由 curl 连接 `downloads.claude.ai:443` 超时；宿主机绕过代理直连同样超时。
- E-002（supports H-001）：宿主代理变量指向 `http://127.0.0.1:7890`；Clash 仅监听回环地址，配置为 `allow-lan: false`。
- E-003（falsifies H-002）：宿主机保持现有代理环境时，请求 Claude bootstrap 与 npm registry 均立即获得 HTTP 200。
- E-004（supports H-001）：运行中的 trial 容器没有 `HTTP_PROXY`、`HTTPS_PROXY`、`ALL_PROXY` 或对应小写变量。
- E-005（supports H-001）：容器直连 Claude 下载站在 4 秒连接超时；访问容器自身 `127.0.0.1:7890` 和 Compose 网关 `172.19.0.1:7890` 均立即拒绝连接。
- E-006（supports H-001）：临时 `socat` 仅监听 `172.17.0.1:17890` 并转发到 `127.0.0.1:7890` 后，来自两个不同 Compose bridge 的 trial 容器均在约 0.5 秒获得 Claude bootstrap HTTP 200。
- E-007（fix-validation）：用户级 `ornnlab-clash-docker-proxy.service` 已启用并运行，监听地址严格为 `172.17.0.1:17890`；当前 Claude Agent Profile 已加入大小写代理变量与本地 `NO_PROXY`。
- E-008（fix-validation）：全新临时 Docker 容器仅通过注入的 `HTTPS_PROXY=http://172.17.0.1:17890` 请求 Claude bootstrap，获得 HTTP 200。
- E-009（fix-validation）：在失败 Job 使用过的同类 terminal-bench task 镜像中，按 Harbor Agent setup 顺序执行 apt、Claude bootstrap 和 `claude --version`，安装成功并返回 Claude Code `2.1.214`；未调用模型 API。

## Root Cause

宿主外网访问依赖 Clash 显式代理，但 Clash 只监听宿主回环地址。Harbor 0.13.2
不会把宿主代理变量自动传入 trial，且容器中的 `127.0.0.1` 不属于宿主。缺少
“Docker 可达的受限代理入口”和“Agent 执行期代理注入”两个条件共同造成失败。

## Fix

- 使用用户级 systemd 管理 `socat`，仅在 Docker 接口 `172.17.0.1:17890`
  提供到 Clash `127.0.0.1:7890` 的转发，不开启 Clash 全局 LAN 访问。
- 在 `claude-code-deepseek-v4-pro` Agent Profile 中配置 HTTP/HTTPS 代理的大小写
  变量和 `NO_PROXY`，使后续 Job 的 Claude 安装与运行阶段使用该入口。
- 将拓扑、验证步骤与 Job 快照边界记录到开发运维 playbook。

## Validation

- systemd service：active (running)，并已 enable。
- 监听面：仅 `172.17.0.1:17890`，未绑定 WLAN 或 `0.0.0.0`。
- 容器端真实请求：Claude bootstrap HTTP 200。
- Profile 映射：六个代理相关变量已保存；未输出或改写原认证凭证。
- Agent setup 冒烟：apt、curl bootstrap、Claude Code 版本检查全部通过。
