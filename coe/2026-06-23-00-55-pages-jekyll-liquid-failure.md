# Problem P-001: GitHub Pages Jekyll build sends failure emails
- Status: fixed
- Created: 2026-06-23 00:55
- Updated: 2026-06-23 00:55
- Objective: Stop GitHub Pages system builds from failing on repository documentation after automatic CI was disabled.
- Symptoms:
  - The user still receives GitHub email for `pages build and deployment / build` failures.
- Expected behavior:
  - Pushes should not fail because Pages parses source documentation as Jekyll/Liquid templates.
- Actual behavior:
  - GitHub Pages legacy build runs against the repository root and fails during Jekyll rendering.
- Impact:
  - Repeated failure emails and noisy GitHub Actions history after CI auto triggers were disabled.
- Reproduction:
  - Push to `main` while GitHub Pages source is configured as `main` branch path `/`.
- Environment:
  - Repository: ceasarXuu/HarnessLab
  - Branch: main
  - Local commit before repair: c46e881f1577887e20341b0a632cf4db49e48710
  - GitHub Pages: legacy branch deploy from `main:/`
- Known facts:
  - E-001: GitHub Pages is configured as legacy build from `main:/`.
  - E-002: The failing system workflow invokes Jekyll with `source: .`.
  - E-003: Jekyll fails on Liquid syntax inside a Markdown code sample.
  - E-004: `.nojekyll` is present at the source root but legacy Pages still invoked Jekyll for the next queued build.
  - E-005: `_config.yml` excludes source docs and diagnostic artifacts from Jekyll rendering.
- Ruled out:
  - The repository `CI` workflow is not the source of these emails; recent failures are from `pages-build-deployment`.
- Fix criteria:
  - A push to `main` includes `.nojekyll`.
  - A push to `main` includes `_config.yml` exclusions for source docs and diagnostic artifacts.
  - No new `CI` push workflow is introduced.
- Current conclusion: The root cause is GitHub Pages legacy Jekyll processing over the repository root, not the project CI workflow.
- Related hypotheses:
  - H-001
  - H-002
- Resolution basis:
  - H-001 confirmed by E-001, E-002, and E-003; repair validated locally by E-004 and remotely by the post-push Pages result.
- Close reason:
  - fixed

## Hypothesis H-002: Pages deploy can fail transiently after a successful build
- Status: confirmed
- Parent: P-001
- Claim: A later `pages-build-deployment` failure for commit `8bb46753c88adcfa04c9de144294ca836e2f65fe` was not caused by source rendering or repository content; the build artifact was valid and GitHub Pages deploy returned a transient service-side `try again later` failure.
- Layer: sub-cause
- Factor relation: independent
- Depends on:
  - none
- Rationale:
  - The failed run's `build` and `report-build-status` jobs succeeded; only `deploy` failed.
- Falsifiable predictions:
  - If true: failed logs should show `actions/deploy-pages@v5` failing after artifact metadata is found and deployment is created, without Jekyll/Liquid errors.
  - If false: failed logs would show a source build, artifact upload, or Jekyll rendering error before deployment.
- Diagnostic evidence plan:
  - Prediction or clause under test: deploy failure is transient and recoverable by rerunning failed jobs.
  - Signal: failed deploy log, rerun status, Pages latest build status.
  - Capture method: `gh run view 28622125030 --log-failed`, `gh run rerun 28622125030 --failed`, `gh api repos/ceasarXuu/HarnessLab/pages/builds/latest`.
  - Event name or marker:
    - pages-build-deployment
  - Correlation keys:
    - run_id=28622125030
    - commit=8bb46753c88adcfa04c9de144294ca836e2f65fe
  - Differentiates from:
    - Jekyll/Liquid source parsing failure.
    - Invalid Pages artifact.
  - Supports if:
    - The failed log reports only deploy failure, then the same run succeeds after rerunning failed jobs.
  - Refutes if:
    - Repeated reruns fail with deterministic source or artifact errors.
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-006
  - E-007
  - E-008
- Conclusion: confirmed
- Repair design readiness: no code repair required
- Next step: no repository code change; rerun failed Pages jobs if this transient deploy error recurs.
- Blocker:
  - none
- Close reason:
  - recovered by rerun

## Hypothesis H-001: Pages legacy Jekyll parses repository docs as Liquid
- Status: confirmed
- Parent: P-001
- Claim: The failure emails continue because GitHub Pages is still active in legacy branch mode and Jekyll parses Markdown documentation containing literal `{{ ... }}` snippets.
- Layer: root-cause
- Factor relation: all_of
- Depends on:
  - none
- Rationale:
  - The email subject names `pages build and deployment`, not `CI`, and the repository has GitHub Pages enabled separately from `.github/workflows/ci.yml`.
- Falsifiable predictions:
  - If true: GitHub Pages config should point at a branch/path source, the failing run should be `pages-build-deployment`, and logs should show a Jekyll/Liquid parse error in repository Markdown.
  - If false: The failing run would come from `.github/workflows/ci.yml` or another explicit workflow rather than the Pages system workflow.
- Diagnostic evidence plan:
  - Prediction or clause under test: GitHub Pages legacy build is the source of the emails and fails while rendering Liquid in Markdown.
  - Signal: GitHub Pages config and failed workflow logs.
  - Capture method: `gh api repos/ceasarXuu/HarnessLab/pages` and `gh run view 27968675281 --log-failed`.
  - Event name or marker:
    - pages-build-deployment
  - Correlation keys:
    - run_id=27968675281
    - commit=12de0c4bacb3755e7cb7a469d805297d8d42f63c
  - Differentiates from:
    - CI workflow auto-trigger still being enabled.
  - Supports if:
    - Pages is configured as legacy `main:/`, the run uses Jekyll source `.`, and the failure is a Liquid syntax error in docs.
  - Refutes if:
    - The run comes from project CI or fails before Jekyll touches repository docs.
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-001
  - E-002
  - E-003
  - E-004
  - E-005
- Conclusion: confirmed
- Repair design readiness: ready
- Next step: push `_config.yml` exclusions and verify the next Pages run outcome.
- Blocker:
  - none
- Close reason:
  - fixed

## Evidence E-001: Pages is configured as legacy branch deploy from repo root
- Related hypotheses:
  - H-001
- Direction: supports
- Type: config
- Source: `gh api repos/ceasarXuu/HarnessLab/pages`
- Prediction or plan link:
  - H-001 diagnostic plan expects Pages source to be branch/path based.
- Matched signal:
  - `build_type=legacy`, `source.branch=main`, `source.path=/`
- Correlation keys:
  - repo=ceasarXuu/HarnessLab
- Raw content:
  ```text
  {"build_type":"legacy","html_url":"https://ceasarxuu.github.io/HarnessLab/","source":{"branch":"main","path":"/"},"status":"errored"}
  ```
- Interpretation: GitHub Pages is a separate active deployment mechanism from the project CI workflow and is watching the repository root.
- Time: 2026-06-23 00:55

## Evidence E-002: Failed system workflow runs Jekyll against source `.`
- Related hypotheses:
  - H-001
- Direction: supports
- Type: diagnostic-log
- Source: `gh run view 27968675281 --log-failed`
- Prediction or plan link:
  - H-001 diagnostic plan expects the failing run to invoke Jekyll over repository root.
- Matched signal:
  - `actions/jekyll-build-pages@v1` with `source: .`
- Correlation keys:
  - run_id=27968675281
- Raw content:
  ```text
  Run actions/jekyll-build-pages@v1
  source: .
  destination: ./_site
  ```
- Interpretation: The failed build is GitHub Pages' Jekyll pipeline, not application CI.
- Time: 2026-06-23 00:55

## Evidence E-003: Failure is Liquid parsing of Markdown source docs
- Related hypotheses:
  - H-001
- Direction: supports
- Type: diagnostic-log
- Source: `gh run view 27968675281 --log-failed`
- Prediction or plan link:
  - H-001 diagnostic plan expects a Liquid syntax error in docs.
- Matched signal:
  - Liquid exception in `docs/releases/v0.1.4/bugfix/04-sse-stream-not-realtime.md`
- Correlation keys:
  - run_id=27968675281
- Raw content:
  ```text
  Liquid Exception: Liquid syntax error (line 107): Variable '{{"status": "{state['experiment']['status']}' was not properly terminated with regexp: /\}\}/ in docs/releases/v0.1.4/bugfix/04-sse-stream-not-realtime.md
  ```
- Interpretation: The direct failure mechanism is Jekyll treating code sample braces as Liquid syntax.
- Time: 2026-06-23 00:55

## Evidence E-004: Root `.nojekyll` did not stop legacy Jekyll build
- Related hypotheses:
  - H-001
- Direction: neutral
- Type: fix-validation
- Source: `.nojekyll` and `gh run view 27968933528 --log-failed`
- Prediction or plan link:
  - H-001 repair expected Pages to stop Jekyll/Liquid processing when `.nojekyll` is present at the source root.
- Matched signal:
  - Root `.nojekyll` exists remotely, but the next legacy Pages run still invoked Jekyll and failed while rendering source Markdown.
- Correlation keys:
  - path=.nojekyll
  - run_id=27968933528
- Raw content:
  ```text
  Remote .nojekyll exists at main with size=1.
  Run actions/jekyll-build-pages@v1
  source: .
  Liquid Exception: Liquid syntax error (line 139) in coe/2026-06-23-00-55-pages-jekyll-liquid-failure.md
  ```
- Interpretation: `.nojekyll` alone is insufficient for this repository's current legacy Pages behavior, so the repair must prevent Jekyll from rendering source documentation through configuration.
- Time: 2026-06-23 00:58

## Evidence E-005: Repair excludes source documentation and diagnostics from Jekyll
- Related hypotheses:
  - H-001
- Direction: supports
- Type: fix-validation
- Source: `_config.yml`
- Prediction or plan link:
  - H-001 repair expects Pages to avoid Liquid parsing failures by excluding non-site source trees from Jekyll rendering.
- Matched signal:
  - `_config.yml` excludes `docs/` and `coe/` from the GitHub Pages Jekyll source scan.
- Correlation keys:
  - path=_config.yml
- Raw content:
  ```text
  exclude:
    - coe/
    - docs/
  ```
- Interpretation: The directories known to contain literal Liquid-like snippets are no longer Jekyll render inputs.
- Time: 2026-06-23 00:58

## Evidence E-006: Latest failed Pages run failed only in deploy
- Related hypotheses:
  - H-002
- Direction: supports
- Type: diagnostic-log
- Source: `gh run view 28622125030 --log-failed`
- Prediction or plan link:
  - H-002 diagnostic plan expects deploy-only failure after a valid artifact.
- Matched signal:
  - `actions/deploy-pages@v5` found the `github-pages` artifact, created a Pages deployment, then returned `Deployment failed, try again later.`
- Correlation keys:
  - run_id=28622125030
  - commit=8bb46753c88adcfa04c9de144294ca836e2f65fe
- Raw content:
  ```text
  Found 1 artifact(s)
  Created deployment for 8bb46753c88adcfa04c9de144294ca836e2f65fe
  ##[error]Deployment failed, try again later.
  ```
- Interpretation: The source build and artifact creation path succeeded; the failure was in the GitHub Pages deployment service step.
- Time: 2026-07-03 05:20

## Evidence E-007: Rerunning failed jobs recovered the Pages workflow
- Related hypotheses:
  - H-002
- Direction: supports
- Type: fix-validation
- Source: `gh run rerun 28622125030 --failed` and `gh run view 28622125030 --json status,conclusion,jobs`
- Prediction or plan link:
  - H-002 diagnostic plan expects the same run to recover when failed deploy is rerun.
- Matched signal:
  - Rerun deploy job completed successfully and the workflow conclusion became `success`.
- Correlation keys:
  - run_id=28622125030
  - deploy_job_id=84880740017
- Raw content:
  ```text
  {"conclusion":"success","status":"completed"}
  deploy / Deploy to GitHub Pages: success
  ```
- Interpretation: No source-code repair was needed for this failure; GitHub accepted the same artifact on retry.
- Time: 2026-07-03 05:20

## Evidence E-008: Pages latest build is now built for the same commit
- Related hypotheses:
  - H-002
- Direction: supports
- Type: fix-validation
- Source: `gh api repos/ceasarXuu/HarnessLab/pages/builds/latest`
- Prediction or plan link:
  - H-002 diagnostic plan expects Pages latest build to show built after the retry.
- Matched signal:
  - Latest Pages build reports `status=built`, `error.message=null`, `commit=8bb46753c88adcfa04c9de144294ca836e2f65fe`.
- Correlation keys:
  - build_id=1075847715
  - commit=8bb46753c88adcfa04c9de144294ca836e2f65fe
- Raw content:
  ```text
  {"status":"built","error":{"message":null},"commit":"8bb46753c88adcfa04c9de144294ca836e2f65fe"}
  ```
- Interpretation: GitHub Pages deployment state recovered for the commit shown in the failure email.
- Time: 2026-07-03 05:20
