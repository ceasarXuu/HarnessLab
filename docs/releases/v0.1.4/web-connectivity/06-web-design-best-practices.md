# BUG-WEB-06 Web 设计最佳实践缺口修复

- Created: 2026-06-25
- Updated: 2026-06-26
- Version: 1.0
- Status: Implemented
- Owner / Responsible: project maintainer
- Related Systems: frontend (Vue 3 + Vite), Vue components, global CSS, i18n/theme runtime
- Related Links: [web-connectivity/README](README.md), [frontend/src/styles.css](../../../../frontend/src/styles.css), [frontend/src/components/AppShell.vue](../../../../frontend/src/components/AppShell.vue), [frontend/src/components/StatePanel.vue](../../../../frontend/src/components/StatePanel.vue), [frontend/index.html](../../../../frontend/index.html)
- Risk Level: Medium
- Plan Type: Standard
- Revision Notes: v1.0 完成 R1-R7 收口：可访问性基线、响应式安全区、i18n/theme 元信息、readonly composable、AppShell live posture composable、Leaderboard 语义表格试点、Storybook smoke 覆盖。

## 背景

v0.1.4 Web 服务调通后，前端已经具备真实 API 数据接入、加载/错误/空态和基础 e2e smoke。但按 Vue Best Practices 与 Web Interface Guidelines 审查，当前控制台仍存在一组用户可感知的设计与可访问性缺口：

1. 交互元素缺少统一 `:focus-visible` 样式，键盘用户难以定位当前焦点。
2. 异步 loading/error 状态缺少 `aria-live` / `role="status"` / `role="alert"`，读屏用户无法及时获知状态变化。
3. 页面缺少 skip link 与 main content anchor，键盘用户每次进入页面都必须重复经过 header/sidebar 导航。
4. 多语言切换未同步 `<html lang>`，浏览器翻译、读屏语音和搜索语义可能不准确。
5. 长文本、窄屏、safe area、touch interaction 等响应式细节尚未形成基线。
6. Route view 承担数据编排与复杂 UI 区块，后续继续扩展会增加维护成本。
7. composable 暴露可变 ref，外部调用方可绕过显式 action 修改状态；整体替换型复杂状态是否使用 `shallowRef()` 需按数据形态判断。

本立项作为 v0.1.4 `web-connectivity` 收尾后的 Web 体验质量修复，不新增业务功能，只修复已识别的设计最佳实践缺口。

## 问题总览

| # | 级别 | 类型 | 涉及文件 | 当前处理建议 |
|---|------|------|----------|--------------|
| 06-A | 高 | 可访问性 | `frontend/src/styles.css`, `AppShell.vue`, `StatePanel.vue` | 补齐 focus-visible、skip link、main anchor、aria-live/alert |
| 06-B | 中 | 响应式 / 触控 | `frontend/src/styles.css` | 处理 safe area、横向溢出、长文本换行、touch-action |
| 06-C | 中 | i18n / theme | `frontend/index.html`, `frontend/src/i18n/*`, `useTheme.ts` | 同步 `<html lang>`，补 theme-color 策略，保留 `color-scheme` |
| 06-D | 中 | Vue 状态模型 | `useTheme.ts`, `useGithubStars.ts`, `views/*.vue`, `AppShell.vue` | composable 返回 readonly state + actions；整体替换型复杂状态再评估是否使用 `shallowRef` |
| 06-E | 中 | 组件边界 | `views/*.vue`, `AppShell.vue` | 将 route view 的数据编排/展示区块拆分到 composable 或 focused components |
| 06-F | 中 | 语义化数据展示 | `AgentsView.vue`, `LeaderboardView.vue`, `DashboardView.vue` | 对表格型数据使用 `<table>` 或补充明确 ARIA 语义 |
| 06-G | 低 | Storybook / 视觉回归 | `frontend/src/components/*.stories.ts` | 为 header、state panel、核心表格/卡片状态补 stories |

## 产品目标

- 键盘用户可以清楚看到当前焦点，并能跳过重复导航直达主内容。
- 读屏用户可以感知加载、错误、空态和重试结果。
- 中英文切换后，页面语言元信息与可见语言一致。
- 在移动端、安全区屏幕和长文本数据下，页面不出现非预期横向滚动或布局撑破。
- Vue 组件职责更清晰，后续新增页面或状态时不继续扩大 route view 复杂度。

## 非目标

- 不新增业务页面、业务筛选、创建/编辑流程。
- 不改变后端 API 契约。
- 不处理生产部署形态、CDN、静态托管策略。
- 不引入大型 UI 框架。
- 不把本修复扩展为完整设计系统重做。

## 修复范围

### R1 可访问性基线

- 为 `.nav-link`、`.header-control`、`.btn` 等交互元素补齐 `:focus-visible` 样式。
- 在应用顶部提供 skip link，跳转到 `<main id="main-content">`。
- 给 loading/idle 状态增加 `role="status"` 或 `aria-live="polite"`。
- 给 error 状态增加 `role="alert"` 或等效可访问提示。
- 装饰性状态点补 `aria-hidden="true"`。
- Retry 按钮补 `type="button"`。

### R2 响应式与交互细节

- 审视并修复窄屏横向溢出；必要时在 app/container 层补 `overflow-x: hidden`。
- 对长 agent name、experiment name、target、score 等文本设置 `overflow-wrap` / `break-words` / truncate 策略。
- 为 full-screen/sticky layout 补 safe-area padding。
- 为交互控件补 `touch-action: manipulation`。
- 为标题补 `text-wrap: balance` 或 `text-wrap: pretty`。

### R3 i18n 与主题元信息

- 初始化阶段调用现有 `setLocale()`，由统一入口负责同步 `vue-i18n` locale、持久化偏好与 `document.documentElement.lang`，避免在多个文件重复实现 lang 同步逻辑。
- locale 切换时继续走 `setLocale()`，确保 `<html lang>` 与当前 locale 一致。
- 评估并补充 `<meta name="theme-color">`，确保亮/暗主题浏览器 UI 颜色与页面背景一致。
- 保持 `:root[data-theme='dark'] { color-scheme: dark; }` 行为不倒退。

### R4 Vue 状态与 composable 数据流

- primitive 状态无需因性能原因强制改为 `shallowRef()`；本期重点是用 `readonly()` 封装对外暴露的状态，避免调用方绕过 action 修改。
- 对整体替换型复杂 UI 状态再单独评估是否使用 `shallowRef()`，优先覆盖：
  - `views/*.vue` 的 `AsyncState`
  - `AppShell.vue` 的 summary state
- `useTheme.ts`、`useGithubStars.ts` 等 composable 对外返回 `readonly()` state，保留显式 actions。
- 如果保留普通 `ref()`，需满足：状态为 primitive 或需要深层响应；并且外部只能通过 action 修改。
- 不为无法发生的异常添加 defensive fallback；仅在外部 API、storage、DOM 边界保留必要防护。

### R5 组件边界整理

- 保持 `App.vue` 作为纯 composition surface。
- 将 `AppShell.vue` 的 live posture 数据逻辑抽出为 `useLivePostureSummary()` 或等价 feature composable。
- 对 route view 中 3+ UI 区块的页面进行拆分，优先级：
  1. `DashboardView.vue`
  2. `AgentsView.vue`
  3. `LeaderboardView.vue`
- 拆分后保持 props down / events up，不引入隐式全局状态。

### R6 语义化数据展示

- 分步推进表格语义化，避免一次性重写多个页面导致视觉与测试风险叠加。
- 第一批以 `LeaderboardView.vue` 作为 `<table>` 试点：它字段少、排序含义明确、验收成本最低。
- 试点通过后，再迁移 `AgentsView.vue`；`DashboardView.vue` 中的实验焦点列表视试点结果决定是否跟进。
- 若继续使用 card/grid，需要提供等效可访问名称、分组语义和键盘阅读顺序。
- 数字列保持 `font-variant-numeric: tabular-nums`。
- 日期、数字格式继续使用 `Intl.*` 或 mapper 层统一格式，不在模板中硬编码格式。

### R7 Storybook 与回归覆盖

- 为以下组件/状态补充或扩展 stories：
  - `AppHeader`：中英文、亮/暗主题、GitHub loading/loaded
  - `StatePanel`：loading/error/empty/idle/ready
  - 关键数据展示组件：长文本、空数据、窄容器
- 保持现有 `storybook:test` 可运行。

## 执行顺序

```text
Phase 0: 现状确认
  - 复核审查清单与当前代码差异
  - 不修改业务行为

Phase 1: 可访问性基线
  - R1 focus-visible / skip link / aria-live / retry button type
  - 先行合并，降低用户可感知风险

Phase 2: 响应式 + i18n/theme 元信息
  - R2 长文本、safe-area、touch-action
  - R3 html lang + theme-color

Phase 3: Vue 状态与组件边界
  - R4 readonly composable state；复杂整体替换状态再评估 shallowRef
  - R5 route view / AppShell 逻辑拆分

Phase 4: 语义化与 Storybook
  - R6 LeaderboardView 表格语义化试点，通过后再迁移 Agents/Dashboard
  - R7 stories + storybook smoke
```

## 验收标准

- [x] 键盘 Tab 导航时，header controls、sidebar links、retry button 均有清晰可见的 `:focus-visible` 样式。（全局 `:focus-visible`，见 [frontend/src/styles.css](../../../../frontend/src/styles.css)）
- [x] 页面提供 skip link，触发后焦点进入 `main#main-content`。（见 [frontend/src/App.vue](../../../../frontend/src/App.vue)、[frontend/src/components/AppShell.vue](../../../../frontend/src/components/AppShell.vue)，并由 e2e 覆盖）
- [x] loading/idle/error 状态具备读屏可感知的 live region 或 alert 语义。（见 [frontend/src/components/StatePanel.vue](../../../../frontend/src/components/StatePanel.vue)、[frontend/src/components/StatePanel.test.ts](../../../../frontend/src/components/StatePanel.test.ts)）
- [x] 切换中英文后，`document.documentElement.lang` 与当前 locale 一致，且初始化与切换均通过统一 `setLocale()` 路径完成。（见 [frontend/src/i18n/index.ts](../../../../frontend/src/i18n/index.ts)、[frontend/src/i18n/i18n.test.ts](../../../../frontend/src/i18n/i18n.test.ts)）
- [x] 亮/暗主题切换后，`color-scheme` 与 `theme-color` 不出现明显倒退。（见 [frontend/index.html](../../../../frontend/index.html)、[frontend/src/composables/useTheme.ts](../../../../frontend/src/composables/useTheme.ts)、[frontend/src/composables/useTheme.test.ts](../../../../frontend/src/composables/useTheme.test.ts)）
- [x] 320px 宽度下无非预期横向滚动；长 agent/experiment 文本不会撑破布局。（CSS 基线 + e2e 断言 `scrollWidth <= innerWidth`）
- [x] `useTheme`、`useGithubStars` 等 composable 不再暴露可随意外部写入的 mutable state。（见 [frontend/src/composables/useTheme.ts](../../../../frontend/src/composables/useTheme.ts)、[frontend/src/composables/useGithubStars.ts](../../../../frontend/src/composables/useGithubStars.ts)）
- [x] composable readonly 行为有测试或类型级检查：调用方不能直接给返回的 state 赋值，只能通过 action 修改。（见 [frontend/src/composables/useTheme.test.ts](../../../../frontend/src/composables/useTheme.test.ts)、[frontend/src/composables/useGithubStars.test.ts](../../../../frontend/src/composables/useGithubStars.test.ts)）
- [x] primitive 状态不因本计划被强制迁移到 `shallowRef()`；若迁移，必须说明它是整体替换型复杂状态或 opaque payload。（primitive 保持 `ref()`；`useLivePostureSummary()` 中 fetched array / error root replacement 使用 `shallowRef()`）
- [x] 至少 Agent 或 Leaderboard 的表格型数据使用语义化 `<table>`，或文档化保留 grid/card 的可访问性理由。（[frontend/src/views/LeaderboardView.vue](../../../../frontend/src/views/LeaderboardView.vue) 已使用 `<table>`）
- [x] 新增/更新 Storybook stories 覆盖 header、state panel、长文本/窄屏状态。（见 [frontend/src/components/AppHeader.stories.ts](../../../../frontend/src/components/AppHeader.stories.ts)、[frontend/src/components/StatePanel.stories.ts](../../../../frontend/src/components/StatePanel.stories.ts)、[frontend/src/components/KpiCard.stories.ts](../../../../frontend/src/components/KpiCard.stories.ts)）
- [x] `npm --prefix frontend run typecheck` 通过。
- [x] `npm --prefix frontend run lint` 通过。
- [x] `npm --prefix frontend run test` 通过。
- [x] `npm --prefix frontend run storybook:test` 通过。
- [x] 若修改 e2e 可见行为，`npm --prefix frontend run e2e` 通过或记录后端不可用时的 conditional skip 证据。

## 测试与日志建设

- 单测：覆盖 composable readonly 行为、`setLocale()` 初始化/切换 lang 同步、GitHub stars 格式化。
- 组件测试：覆盖 `StatePanel` live region、retry emit、header controls aria-label。
- e2e smoke：覆盖键盘导航、skip link、主路由首屏可见。
- Storybook smoke：覆盖视觉状态可启动。
- 日志/可观测性：本修复不新增业务日志，因为变更集中在前端可访问性、展示语义、组件边界与 Storybook 覆盖；外部 API 失败仍由现有 `StatePanel` 用户可见错误态承接，不记录敏感数据。

## Implementation Summary

| 范围 | 落地内容 | 证据 |
|---|---|---|
| R1 可访问性 | skip link、`main#main-content`、全局 `:focus-visible`、StatePanel live region / alert、retry `type="button"`、装饰状态点 `aria-hidden` | `App.vue`、`AppShell.vue`、`StatePanel.vue`、`StatePanel.test.ts`、`navigation.spec.ts` |
| R2 响应式 / 触控 | `overflow-x` 基线、safe-area padding、`touch-action`、长文本 `overflow-wrap`、header 320px 换行 | `styles.css`、e2e 320px overflow 断言 |
| R3 i18n / theme | `initializeLocale()` 统一入口、`html lang` 同步、`theme-color` 随主题切换 | `i18n/index.ts`、`main.ts`、`index.html`、`useTheme.ts` |
| R4 Vue 状态 | `useTheme` / `useGithubStars` readonly state；`useLivePostureSummary` 对整体替换型数组/错误状态使用 `shallowRef()` | composable 单测 |
| R5 组件边界 | `AppShell` 的 live posture API 副作用抽出为 `useLivePostureSummary()` | `AppShell.vue`、`useLivePostureSummary.ts` |
| R6 语义化数据 | `LeaderboardView` 使用真实 `<table>`、`th scope="col"`、数字列 tabular nums | `LeaderboardView.vue`、`views.test.ts` |
| R7 Storybook | AppHeader 与 StatePanel stories；Storybook preview 安装 i18n | `.storybook/preview.ts`、`AppHeader.stories.ts`、`StatePanel.stories.ts` |

## 验证记录

- `npm --prefix frontend run typecheck`：通过
- `npm --prefix frontend run lint`：通过
- `npm --prefix frontend run test`：100 tests 通过
- `npm --prefix frontend run storybook:test`：通过
- `npm --prefix frontend run e2e`：通过（含 skip link 与 320px overflow smoke）

## 风险与约束

- 表格语义化可能影响现有视觉布局，需要用 CSS 保持当前控制台视觉风格。
- `readonly()` composable state 会要求调用方通过 action 修改状态，可能需要同步调整测试。
- primitive 状态不做强制 `shallowRef()` 迁移，避免为了形式合规引入无收益改动。
- `theme-color` 的动态切换需避免与浏览器兼容性问题耦合过深；本期以稳定元信息为准。
- Route view 拆分不得引入过度抽象；只拆分已有明确 UI 区块和可复用状态逻辑。

## Implementation Notes

- 当前项目使用 Vue 3.5.x，允许采用 `readonly()`、`useTemplateRef()` 等现代 Vue API；`shallowRef()` 仅用于整体替换型复杂状态或 opaque payload，不作为 primitive 状态的默认要求。
- 继续遵循 `<script setup lang="ts">`。
- 继续遵循 Storybook 工作流，新增 UI 状态应尽量先有 story 或同步补 story。
- 每一批代码修复完成后需运行对应 typecheck/lint/test，并根据用户要求决定是否执行对抗性审查。
