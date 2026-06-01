use askama::Template;
use harnesslab_core::{
    AttemptProvenance, FailureClass, RunResults, TaskAttemptResult, UsageRecord,
    report_artifact_path, task_dir_name,
};

#[derive(Debug, Clone)]
pub struct ReportModel {
    pub title: String,
    pub run_id: String,
    pub agent: String,
    pub benchmark: String,
    pub split: String,
    pub report_path: String,
    pub resumed: bool,
    pub summary: harnesslab_core::RunSummary,
    pub run_health_status: String,
    pub run_health_reason: String,
    pub has_run_health_reason: bool,
    pub rows: Vec<TaskRow>,
    pub total_usage: String,
    pub agent_config_summary: String,
    pub replay_command: String,
    pub original_command: String,
}

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub task_id: String,
    pub attempt: u32,
    pub resumed_marker: String,
    pub outcome: String,
    pub failure: String,
    pub score: String,
    pub duration_ms: u64,
    pub usage: String,
    pub patch_href: String,
    pub has_patch: bool,
    pub stdout_link: String,
    pub stderr_link: String,
    pub verifier_stdout_link: String,
    pub verifier_stderr_link: String,
    pub warnings: String,
    pub has_warnings: bool,
}

#[derive(Debug, Clone)]
pub struct ReportContext {
    pub run_id: String,
    pub agent: String,
    pub agent_config_summary: String,
    pub benchmark: String,
    pub split: String,
    pub report_path: String,
    pub replay_command: String,
    pub original_command: String,
    pub resumed: bool,
    pub run_health_status: String,
    pub run_health_reason: String,
}

#[derive(Template)]
#[template(
    source = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{{ model.title }}</title>
  <style>
    body { font-family: system-ui, sans-serif; margin: 32px; color: #17202a; }
    table { border-collapse: collapse; width: 100%; }
    th, td { border: 1px solid #ccd1d1; padding: 6px 8px; text-align: left; }
    th { background: #eef2f3; }
    code { background: #eef2f3; padding: 2px 4px; }
  </style>
</head>
<body>
  <h1>{{ model.title }}</h1>
  <p>Run <code>{{ model.run_id }}</code> using {{ model.agent }} on {{ model.benchmark }} / {{ model.split }}.</p>
  <p>Report: <code>{{ model.report_path }}</code></p>
  <p>Resume: {% if model.resumed %}yes{% else %}no{% endif %}</p>
  <p>Run health: <strong>{{ model.run_health_status }}</strong>{% if model.has_run_health_reason %}: {{ model.run_health_reason }}{% endif %}. <a href="run-health.json">run-health.json</a></p>
  <p>Agent config: {{ model.agent_config_summary }}. Snapshot: <a href="agent-profile.snapshot.json">agent-profile.snapshot.json</a>, command: <a href="command.txt">command.txt</a>.</p>
  <h2>Summary</h2>
  <p>Total {{ model.summary.total_tasks }}, success {{ model.summary.success }}, benchmark failures {{ model.summary.benchmark_failure }}, execution failures {{ model.summary.execution_failure }}, interrupted {{ model.summary.interrupted }}, score {{ model.summary.total_score }}.</p>
  <p>Usage: {{ model.total_usage }}</p>
  <p>Score uses the latest attempt per task. Usage sums all recorded attempts. Usage marked unknown is cost not comparable.</p>
  <h2>Tasks</h2>
  <table>
    <thead><tr><th>Task</th><th>Attempt</th><th>Resume</th><th>Outcome</th><th>Failure</th><th>Warnings</th><th>Score</th><th>Duration</th><th>Usage</th><th>Patch</th><th>Logs</th></tr></thead>
    <tbody>
    {% for row in model.rows %}
      <tr>
        <td>{{ row.task_id }}</td>
        <td>{{ row.attempt }}</td>
        <td>{{ row.resumed_marker }}</td>
        <td>{{ row.outcome }}</td>
        <td>{{ row.failure }}</td>
        <td>{% if row.has_warnings %}{{ row.warnings }}{% else %}none{% endif %}</td>
        <td>{{ row.score }}</td>
        <td>{{ row.duration_ms }} ms</td>
        <td>{{ row.usage }}</td>
        <td>{% if row.has_patch %}<a href="{{ row.patch_href }}">diff</a>{% else %}n/a{% endif %}</td>
        <td><a href="{{ row.stdout_link }}">agent stdout</a> <a href="{{ row.stderr_link }}">agent stderr</a> <a href="{{ row.verifier_stdout_link }}">verifier stdout</a> <a href="{{ row.verifier_stderr_link }}">verifier stderr</a></td>
      </tr>
    {% endfor %}
    </tbody>
  </table>
  <h2>Reproduce</h2>
  <p>Replay: <code>{{ model.replay_command }}</code></p>
  <p>Original: <code>{{ model.original_command }}</code></p>
</body>
</html>"#,
    ext = "html"
)]
struct HtmlTemplate<'a> {
    model: &'a ReportModel,
}

pub fn build_report_model(context: ReportContext, results: RunResults) -> ReportModel {
    let rows = results
        .tasks
        .iter()
        .map(|task| {
            let failure = match task.failure_class {
                FailureClass::None => "none".to_string(),
                class => format!(
                    "{}/{}",
                    debug_snake(class),
                    task.failure_code
                        .map(debug_snake)
                        .unwrap_or("none".to_string())
                ),
            };
            let patch_href = patch_href(task);
            let warnings = warnings_text(&task.warnings);
            TaskRow {
                task_id: task.task_id.clone(),
                attempt: task.attempt,
                resumed_marker: provenance_text(task.provenance).to_string(),
                outcome: format!("{:?}", task.outcome),
                failure,
                score: format!("{:.3}", task.benchmark_score),
                duration_ms: task.duration_ms,
                usage: usage_text(&task.usage),
                has_patch: patch_href != "n/a",
                patch_href,
                stdout_link: format!(
                    "tasks/{}/attempts/{}/agent/stdout.log",
                    task_dir_name(&task.task_id).unwrap_or_else(|_| "_invalid-task-id".to_string()),
                    task.attempt
                ),
                stderr_link: format!(
                    "tasks/{}/attempts/{}/agent/stderr.log",
                    task_dir_name(&task.task_id).unwrap_or_else(|_| "_invalid-task-id".to_string()),
                    task.attempt
                ),
                verifier_stdout_link: format!(
                    "tasks/{}/attempts/{}/verifier/stdout.log",
                    task_dir_name(&task.task_id).unwrap_or_else(|_| "_invalid-task-id".to_string()),
                    task.attempt
                ),
                verifier_stderr_link: format!(
                    "tasks/{}/attempts/{}/verifier/stderr.log",
                    task_dir_name(&task.task_id).unwrap_or_else(|_| "_invalid-task-id".to_string()),
                    task.attempt
                ),
                has_warnings: !warnings.is_empty(),
                warnings,
            }
        })
        .collect();
    ReportModel {
        title: "HarnessLab Run Report".to_string(),
        run_id: context.run_id.clone(),
        agent: context.agent.clone(),
        benchmark: context.benchmark.clone(),
        split: context.split.clone(),
        report_path: context.report_path,
        resumed: context.resumed,
        summary: results.summary,
        run_health_status: context.run_health_status,
        has_run_health_reason: !context.run_health_reason.is_empty(),
        run_health_reason: context.run_health_reason,
        rows,
        total_usage: total_usage_text(&results.tasks),
        agent_config_summary: context.agent_config_summary,
        replay_command: context.replay_command,
        original_command: context.original_command,
    }
}

pub fn render_html(model: &ReportModel) -> Result<String, askama::Error> {
    HtmlTemplate { model }.render()
}

fn usage_text(usage: &UsageRecord) -> String {
    match usage {
        UsageRecord::Unknown => "unknown; cost not comparable".to_string(),
        UsageRecord::ParseError { message } => format!("parse error: {message}"),
        UsageRecord::Parsed {
            total_tokens,
            cost_usd,
            ..
        } => match cost_usd {
            Some(cost) => format!("{total_tokens} tokens; ${cost:.6}"),
            None => format!("{total_tokens} tokens"),
        },
    }
}

fn warnings_text(warnings: &[harnesslab_core::FailureCode]) -> String {
    warnings
        .iter()
        .map(debug_snake)
        .collect::<Vec<_>>()
        .join(", ")
}

fn debug_snake(value: impl std::fmt::Debug) -> String {
    let debug = format!("{value:?}");
    let mut out = String::new();
    for (index, ch) in debug.chars().enumerate() {
        if ch.is_ascii_uppercase() && index > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

fn total_usage_text(tasks: &[TaskAttemptResult]) -> String {
    let mut total = 0;
    let mut cost = 0.0;
    let mut has_cost = false;
    let mut unknown = false;
    for task in tasks {
        match &task.usage {
            UsageRecord::Parsed {
                total_tokens,
                cost_usd,
                ..
            } => {
                total += total_tokens;
                if let Some(value) = cost_usd {
                    has_cost = true;
                    cost += value;
                }
            }
            _ => unknown = true,
        }
    }
    let cost_text = if has_cost {
        format!("; ${cost:.6}")
    } else {
        String::new()
    };
    if total > 0 && !unknown {
        format!("{total} tokens{cost_text}")
    } else if total > 0 {
        format!("{total} tokens{cost_text}; some attempts unknown")
    } else {
        "unknown; cost not comparable".to_string()
    }
}

fn patch_href(task: &TaskAttemptResult) -> String {
    match &task.patch {
        Some(patch) => match (
            task_dir_name(&task.task_id),
            report_artifact_path(&patch.diff_path),
        ) {
            (Ok(task_dir), Ok(diff_path)) => {
                format!("tasks/{task_dir}/attempts/{}/{}", task.attempt, diff_path)
            }
            _ => "n/a".to_string(),
        },
        None => "n/a".to_string(),
    }
}

fn provenance_text(provenance: AttemptProvenance) -> &'static str {
    match provenance {
        AttemptProvenance::Original => "original",
        AttemptProvenance::Resumed => "resumed",
        AttemptProvenance::Recovery => "recovery",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{Outcome, PatchRecord, PatchStatus, TaskAttemptResult, TaskState};

    #[test]
    fn rpt_001_report_html_contains_summary_and_relative_links() {
        let results = harnesslab_core::summarize_results("run-1", vec![attempt()]);
        let model = build_report_model(context(false), results);

        let html = render_html(&model).unwrap();

        assert!(html.contains("HarnessLab Run Report"));
        assert!(html.contains("cost not comparable"));
        assert!(html.contains("Agent config:"));
        assert!(html.contains("Run health:"));
        assert!(html.contains("interrupted 0"));
        assert!(html.contains("<th>Warnings</th>"));
        assert!(html.contains("agent-profile.snapshot.json"));
        assert!(html.contains("command.txt"));
        assert!(html.contains("tasks/task-1/attempts/1/agent/stdout.log"));
        assert!(html.contains("tasks/task-1/attempts/1/verifier/stdout.log"));
    }

    #[test]
    fn rpt_001_report_encodes_task_ids_and_rejects_unsafe_patch_links() {
        let mut result = attempt();
        result.task_id = "task/slash".to_string();
        result.provenance = AttemptProvenance::Recovery;
        result.warnings = vec![harnesslab_core::FailureCode::AgentTimeout];
        result.patch = Some(PatchRecord {
            diff_path: "../patch.diff".to_string(),
            prediction_path: None,
            status: PatchStatus::Captured,
        });
        let results = harnesslab_core::summarize_results("run-1", vec![result]);
        let model = build_report_model(context(true), results);

        let html = render_html(&model).unwrap();

        assert!(html.contains("task/slash"));
        assert!(html.contains("<td>recovery</td>"));
        assert!(html.contains("<td>agent_timeout</td>"));
        assert!(html.contains("tasks/task%2Fslash/attempts/1/agent/stdout.log"));
        assert!(html.contains("<td>n/a</td>"));
        assert!(!html.contains("../patch.diff"));
    }

    fn attempt() -> TaskAttemptResult {
        TaskAttemptResult {
            schema_version: 1,
            task_id: "task-1".to_string(),
            attempt: 1,
            provenance: AttemptProvenance::Original,
            state: TaskState::Success,
            outcome: Outcome::Success,
            failure_class: FailureClass::None,
            failure_code: None,
            health_impact: harnesslab_core::HealthImpact::None,
            benchmark_score: 1.0,
            duration_ms: 1,
            agent: None,
            evaluation: None,
            patch: None,
            usage: UsageRecord::Unknown,
            warnings: Vec::new(),
        }
    }

    fn context(resumed: bool) -> ReportContext {
        ReportContext {
            run_id: "run-1".to_string(),
            agent: "fake".to_string(),
            agent_config_summary:
                "kind=fake; input_mode=stdin; timeout_sec=3600; concurrency=4; attempts=1; network=full"
                    .to_string(),
            benchmark: "fake-terminal".to_string(),
            split: "success".to_string(),
            report_path: "/runs/run-1/report.html".to_string(),
            replay_command: "harnesslab run replay /runs/run-1".to_string(),
            original_command:
                "harnesslab --home /tmp/home run --agent fake --benchmark fake-terminal --split success"
                    .to_string(),
            resumed,
            run_health_status: "ok".to_string(),
            run_health_reason: String::new(),
        }
    }
}
