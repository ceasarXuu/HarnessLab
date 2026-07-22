# OrnnLab Development Operations

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-15 | Recorded operational lessons for the Harbor WebUI rewrite. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Linked operations guidance to document version governance. |
| 1.2 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-27 | Recorded Colima startup check before real Harbor Docker smoke. |
| 1.3 | Python app `0.2.0`; Harbor `0.13.x` | 2026-07-19 | 记录 macOS 到 Ubuntu 的数据恢复与验证经验。 |
| 1.4 | npm launcher `0.1.3`; Vite `8.x` | 2026-07-19 | 记录 Ubuntu inotify 限额导致前端启动失败的诊断和部署方案。 |
| 1.5 | Harbor `0.13.x`; Docker Engine | 2026-07-19 | 记录 Clash 回环代理无法被 Harbor trial 容器访问的诊断和安全转发方案。 |
| 1.6 | Harbor `0.13.x`; Docker Engine | 2026-07-19 | OrnnLab 自动发现宿主代理并为 Docker Agent 托管临时 relay。 |
| 1.7 | Harbor `0.13.x`; Docker Engine | 2026-07-23 | 记录历史 Harbor 容器归属确认、精确回收与跨项目保护流程。 |
| 1.8 | Harbor `0.13.x`; Docker Engine | 2026-07-23 | 记录 OrnnLab 所有权标签、自动回收和多实例隔离验证流程。 |

This file records current operational lessons for the Harbor WebUI rewrite.
Legacy Rust CLI operations were archived on 2026-06-15.

- Archived copy: `../archive/2026-06-15-pre-harbor-webui-redesign/development-operations.md`
- Current plan: `plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

## 2026-06-15 Documentation Archive Pass

When moving old documents, keep the historical content under
`docs/archive/2026-06-15-pre-harbor-webui-redesign/` and leave only short
supersession stubs at old paths that are still referenced by tests, reports, or
onboarding links.

Legacy tests that validate old document semantics should read the archived copy
directly. Current stubs must stay short and must not contain stale
implementation instructions.

## 2026-06-27 Real Harbor Docker Smoke

本机有 Docker CLI 不代表 Docker daemon 已可用。真实 Harbor Docker smoke
前先确认当前 context 和 Colima 状态：

```bash
docker context ls
colima status
docker info
```

如果 `docker info` 报 `/Users/xuzhang/.colima/default/docker.sock` 不存在，
说明当前 `colima` context 指向的 daemon 没启动。先执行：

```bash
colima start
docker info
ORNNLAB_REAL_HARBOR=1 uv run pytest -m docker tests/python/test_harbor_real_smoke.py tests/python/test_real_harbor_cancel_recovery.py -vv
```

2026-06-27 验证结果：`colima start` 后 Docker ServerVersion 为 `29.2.1`，
真实 Harbor Docker smoke `3 passed`，耗时约 3 分 58 秒。

## 2026-07-19 Ubuntu 数据恢复

恢复包的外层 `.sha256` 使用相对文件名，必须在归档所在目录执行校验：

```bash
cd /path/to/backup
sha256sum -c ornnlab-ubuntu-backup-20260719.tar.gz.sha256
```

恢复前还要在解包目录执行 `sha256sum -c SHA256SUMS`，并通过
`git bundle verify payload/HarnessLab.bundle` 校验源码历史。如果当前仓库 HEAD
与 manifest 的 `gitCommit` 一致且工作区干净，可保留当前仓库，仅恢复
`~/.ornnlab/data` 与 `~/ornnlab-data`，避免恢复脚本因源码目录非空而停止。

macOS 创建的归档可能包含 `LIBARCHIVE.xattr.*` 扩展头和 `._*` AppleDouble
文件。扩展头警告不代表内容损坏；解压外置数据时应过滤 AppleDouble 文件：

```bash
tar -xzf payload/external-data.tar.gz \
  -C ~/ornnlab-data \
  --exclude='._*' \
  --exclude='*/._*'
```

完成主数据导入和 `rebase_paths.py` 路径重映射后，至少执行以下验证：

1. 对 `~/.ornnlab/data/*.sqlite` 执行 `PRAGMA integrity_check`。
2. 搜索 `/Users/` 与旧 `/Volumes/` 数据路径，确认没有可识别文本残留。
3. 确认 `terminal-bench@2.0`、`swebenchpro@1.0` 和 Job 目录存在。
4. 执行 `ORNNLAB_HOME=~/.ornnlab/data uv run ornnlab doctor`，确认 Docker、Harbor、数据库 schema 与孤儿容器检查均正常。

## 2026-07-19 Ubuntu Vite watcher 限额

如果 `ornnlab dev start` 报 `frontend exited before becoming ready`，且
`~/.ornnlab/dev-service/logs/frontend.log` 包含以下错误：

```text
ENOSPC: System limit for number of file watchers reached
```

先检查 inotify 限额和当前占用，不要把该错误误判成磁盘空间不足：

```bash
sysctl fs.inotify.max_user_watches fs.inotify.max_user_instances
```

有 sudo 权限时，推荐提高开发机的 watch 限额并持久化：

```bash
sudo tee /etc/sysctl.d/99-ornnlab-inotify.conf >/dev/null <<'EOF'
fs.inotify.max_user_watches=524288
fs.inotify.max_user_instances=512
EOF
sudo sysctl --system
```

暂时不能修改内核参数时，只对 OrnnLab launcher 启用 polling，避免影响其他
Node 项目：

```bash
CHOKIDAR_USEPOLLING=true CHOKIDAR_INTERVAL=500 ornnlab dev start
```

部署到固定源码目录时，可用用户级 wrapper 固化 `ORNNLAB_SOURCE`、
`ORNNLAB_HOME` 和上述 polling 环境。wrapper 应放在用户自己的 `~/.local/bin`
下，不提交硬编码的本机路径到仓库。完成后必须执行 `dev stop`、`dev start`、
`dev status --json` 生命周期回归，并验证后端与前端代理的 `/api/webui/v1/system/live`。

## 2026-07-19 Docker 容器访问 Clash

宿主机能访问 Claude 或 npm，不代表 Harbor trial 容器也能访问。Clash Verge
常见配置是只监听 `127.0.0.1:7890`；该地址进入容器后指向容器自身。Harbor
也不会自动把宿主的代理变量注入 Agent。因此应分别验证宿主经代理请求、容器
代理变量和容器到宿主代理入口三层，不要只用宿主 curl 判断。

当前 OrnnLab 默认自动读取标准代理变量，并先识别有效 Docker target。实现不识别
Clash、Docker Desktop、Colima 等具体产品，也不假定固定安装位置或网段。策略矩阵：

| Docker target / 代理地址 | 自动策略 |
|---|---|
| 任意 target + 非回环代理 URL | 直接注入 Environment；容器网络负责 DNS 与路由可达性 |
| 同主机 rootful Linux + 回环 HTTP/SOCKS | bind host gateway 后创建仅限当前 Job 的 relay |
| Docker Desktop、rootless、远程/虚拟化 daemon + 回环代理 | 启动 Harbor 前明确失败；改用容器可达的 Profile 代理 |
| Agent/Environment 已显式配置某代理组 | 跳过该组的自动读取和 target relay |
| Agent/Environment 配置 `extra_allowed_hosts` | 整体跳过默认自动代理；如确需代理，在 Profile 中显式配置 |

能力选择可通过日志确认：

```text
docker_proxy_detection
docker_proxy_target_classified
docker_proxy_bridge_started
docker_proxy_policy_skipped
docker.proxy.injected
docker_proxy_policy_released
harbor_subprocess.runtime_config_prepared
```

自动代理模板写入 Harbor Environment，因此 setup、Agent 和 Verifier 使用同一策略。
OrnnLab 生成的 `harbor.config.json` 只保存 `${ORNNLAB_CONTAINER_*}` 模板；subprocess runner
在受限临时目录生成已解析配置供本次 Harbor 进程读取，结束或取消后自动清理。日志只记录
解析变量数量，不记录代理 endpoint。Harbor 为恢复 Job 生成的 `lock.json` 和 trial
`config.json` 会按其原生协议快照已解析的 relay 地址；自动模式拒绝带凭据的回环代理，
因此该快照不包含代理认证信息。

临时排障或旧版本仍可用 `socat` 建立仅绑定 Docker 接口的转发：

```ini
[Service]
ExecStart=/usr/bin/socat TCP-LISTEN:17890,bind=172.17.0.1,fork,reuseaddr TCP:127.0.0.1:7890
SuccessExitStatus=143
Restart=always
RestartSec=3
```

旧版本把该命令保存为用户级 systemd service 后，还需在需要联网的 Agent Profile
中配置：

```text
HTTP_PROXY=http://172.17.0.1:17890
HTTPS_PROXY=http://172.17.0.1:17890
http_proxy=http://172.17.0.1:17890
https_proxy=http://172.17.0.1:17890
NO_PROXY=localhost,127.0.0.1,::1
no_proxy=localhost,127.0.0.1,::1
```

上述 `172.17.0.1` 只是一种旧版、本地 rootful Linux 的示例，不能复制到其他设备
作为默认配置。部署前必须从有效 Docker Context 重新确认 gateway 属于运行 OrnnLab
的主机；转发端口只能监听 Docker host gateway，不能监听 `0.0.0.0`。

验证时从临时 Docker 容器发起真实请求，并检查 systemd 监听范围：

```bash
systemctl --user status ornnlab-clash-docker-proxy.service
ss -ltnp 'sport = :17890'
docker run --rm \
  -e HTTPS_PROXY=http://172.17.0.1:17890 \
  curlimages/curl:8.10.1 \
  -fsS -o /dev/null https://downloads.claude.ai/claude-code-releases/bootstrap.sh
```

Agent Profile 是 Job 创建时的快照。修改 Profile 只影响后续新建或重跑的 Job，
不会修复已经运行中的 Harbor 进程；旧 Job 应在确认成本后取消，再基于更新后的
Profile 重跑。

自动继承模式在 Job 执行期读取当前宿主代理，不受上述 Profile 快照限制。需要排除
代理问题或使用自管网络策略时，启动 OrnnLab 前设置：

```bash
ORNNLAB_DOCKER_PROXY_MODE=off ornnlab dev start
```

启停回归判断端口是否仍被服务占用时，应实际尝试 TCP connect，不应立即重新 bind
同一端口。刚关闭的健康检查连接可能处于 `TIME_WAIT`，此时没有监听进程，但 bind
仍会短暂返回 `EADDRINUSE`，从而产生“服务未退出”的假失败。

Storybook 启动时报 `ENOSPC: System limit for number of file watchers reached` 表示当前
用户的 inotify 配额被 IDE、开发服务器等进程共同耗尽，不是前端构建失败。先用
`fs.inotify.max_user_instances`、`fs.inotify.max_user_watches` 和 `/proc/*/fd` 定位，
不要直接终止不属于当前任务的进程。一次性 CI 冒烟可改用 polling，避免修改设备级
sysctl：

```bash
CHOKIDAR_USEPOLLING=true WATCHPACK_POLLING=true npm run storybook:test --prefix frontend
```

## 2026-07-23 历史 Harbor Docker 精确回收

清理历史 Job 容器时禁止直接使用 `docker system prune` 或按通用 Task 名批量删除。
同一 Docker daemon 可能同时服务 OrnnLab、WhaleCode 和其他项目，名称中出现
`terminal-bench`、`regex-log` 等词不足以证明归属。

归属应至少由以下一种强证据确认：

1. 容器 bind mount 的 source 位于当前 OrnnLab 配置的 `jobsDir` 下；
2. Compose 标签 `com.docker.compose.project.config_files` 指向当前 OrnnLab 环境安装的
   Harbor compose 文件；
3. Compose project 与上述 mount、Job 产物目录能够一一对应。

先检查全部候选对象，确认容器均为 stopped/exited，并确认对应网络的容器数为 0：

```bash
docker ps -a --format '{{json .}}'
docker inspect <explicit-container-id>...
docker network inspect <explicit-network-name>...
```

删除时只传入检查过的显式容器 ID 和网络 ID：

```bash
docker rm <explicit-container-id>...
docker network rm <explicit-network-id>...
```

任务镜像与容器是不同对象。`ghcr.io/laude-institute/terminal-bench/*` 镜像是后续 Job
可复用的运行依赖，不能仅因当前没有容器就删除；BuildKit cache 也缺少可靠的 Job
归属标签，不应混入容器回收。清理完成后再次用同一 mount/Compose 归属条件扫描，结果
必须为 0，并用 `docker system df` 与 `df -h /` 记录当前空间，不把同时发生的其他清理
全部归因于本次操作。

2026-07-23 本机验证：精确删除 4 个确认属于 OrnnLab/HarnessLab 的 exited Harbor
容器及 4 个空 Compose 网络，容器可写层约 916 MiB；其余 113 个 stopped 容器均挂载
另一个项目目录，未删除。Terminal-Bench 镜像、跨项目容器和无法证明归属的 dangling
镜像全部保留。

## 2026-07-23 OrnnLab Docker 所有权自动回收

新 Job 不再依赖历史容器的 mount 或安装路径进行事后猜测。`ORNNLAB_HOME` marker 中的
稳定 `instance_id` 与 run ID 在 Harbor Compose 创建前写入每个服务、网络和卷。排障时先执行：

```bash
ornnlab doctor
docker ps -a --filter label=ornnlab.managed=true --format '{{json .}}'
docker network ls --filter label=ornnlab.managed=true
docker volume ls --filter label=ornnlab.managed=true
```

健康报告中的孤儿数只统计当前数据目录实例、非活动 Run 且清理策略不是 `retain` 的容器。
Job 结束后检查事件 `docker.ownership.cleanup_completed`；失败时检查
`docker.ownership.cleanup_failed` 的显式 Docker 错误和服务日志中的
`docker.ownership.cleanup`，不要立刻运行全局 prune。

服务启动会在 Run 状态恢复之后再执行一次幂等对账。若同一 daemon 运行多个 OrnnLab
数据目录，验证过滤条件必须同时包含 `ornnlab.instance_id` 和 `ornnlab.run_id`；只使用
run ID 可能跨实例误删。显式 `keep_containers` 会写入 `ornnlab.cleanup=retain`，这类容器
属于用户要求的保留对象，不应报告为泄漏。

真实冒烟使用随机 project、instance 和 run，至少包含 main 与 sidecar；验证两个容器和
默认网络标签一致、回收计数为 2、对应空网络被删除、同一 run 的二次扫描为 0。测试 finally 仍需
执行同一随机 Compose project 的 `down --volumes --remove-orphans`，确保测试异常时也收敛。
