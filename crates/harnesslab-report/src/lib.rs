use askama::Template;
use harnesslab_core::{FailureClass, RunResults, UsageRecord};

#[derive(Debug, Clone)]
pub struct ReportModel {
    pub title: String,
    pub run_id: String,
    pub agent: String,
    pub benchmark: String,
    pub split: String,
    pub summary: harnesslab_core::RunSummary,
    pub rows: Vec<TaskRow>,
    pub replay_command: String,
    pub original_command: String,
}

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub task_id: String,
    pub outcome: String,
    pub failure: String,
    pub score: String,
    pub duration_ms: u64,
    pub usage: String,
    pub stdout_link: String,
    pub stderr_link: String,
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
  <h2>Summary</h2>
  <p>Total {{ model.summary.total_tasks }}, success {{ model.summary.success }}, benchmark failures {{ model.summary.benchmark_failure }}, execution failures {{ model.summary.execution_failure }}.</p>
  <p>Usage marked unknown is cost not comparable.</p>
  <h2>Tasks</h2>
  <table>
    <thead><tr><th>Task</th><th>Outcome</th><th>Failure</th><th>Score</th><th>Duration</th><th>Usage</th><th>Logs</th></tr></thead>
    <tbody>
    {% for row in model.rows %}
      <tr>
        <td>{{ row.task_id }}</td>
        <td>{{ row.outcome }}</td>
        <td>{{ row.failure }}</td>
        <td>{{ row.score }}</td>
        <td>{{ row.duration_ms }} ms</td>
        <td>{{ row.usage }}</td>
        <td><a href="{{ row.stdout_link }}">stdout</a> <a href="{{ row.stderr_link }}">stderr</a></td>
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

pub fn build_report_model(
    run_id: &str,
    agent: &str,
    benchmark: &str,
    split: &str,
    results: RunResults,
) -> ReportModel {
    let rows = results
        .tasks
        .iter()
        .map(|task| {
            let failure = match task.failure_class {
                FailureClass::None => "none".to_string(),
                class => format!("{class:?}/{:?}", task.failure_code),
            };
            TaskRow {
                task_id: task.task_id.clone(),
                outcome: format!("{:?}", task.outcome),
                failure,
                score: format!("{:.3}", task.benchmark_score),
                duration_ms: task.duration_ms,
                usage: usage_text(&task.usage),
                stdout_link: format!(
                    "tasks/{}/attempts/{}/agent/stdout.log",
                    task.task_id, task.attempt
                ),
                stderr_link: format!(
                    "tasks/{}/attempts/{}/agent/stderr.log",
                    task.task_id, task.attempt
                ),
            }
        })
        .collect();
    ReportModel {
        title: "HarnessLab Run Report".to_string(),
        run_id: run_id.to_string(),
        agent: agent.to_string(),
        benchmark: benchmark.to_string(),
        split: split.to_string(),
        summary: results.summary,
        rows,
        replay_command: format!("harnesslab run replay ~/.harnesslab/runs/{run_id}"),
        original_command: format!(
            "harnesslab run --agent {agent} --benchmark {benchmark} --split {split}"
        ),
    }
}

pub fn render_html(model: &ReportModel) -> Result<String, askama::Error> {
    HtmlTemplate { model }.render()
}

fn usage_text(usage: &UsageRecord) -> String {
    match usage {
        UsageRecord::Unknown => "unknown; cost not comparable".to_string(),
        UsageRecord::ParseError { message } => format!("parse error: {message}"),
        UsageRecord::Parsed { total_tokens, .. } => format!("{total_tokens} tokens"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use harnesslab_core::{Outcome, TaskAttemptResult, TaskState};

    #[test]
    fn rpt_001_report_html_contains_summary_and_relative_links() {
        let results = harnesslab_core::summarize_results("run-1", vec![attempt()]);
        let model = build_report_model("run-1", "fake", "fake-terminal", "success", results);

        let html = render_html(&model).unwrap();

        assert!(html.contains("HarnessLab Run Report"));
        assert!(html.contains("cost not comparable"));
        assert!(html.contains("tasks/task-1/attempts/1/agent/stdout.log"));
    }

    fn attempt() -> TaskAttemptResult {
        TaskAttemptResult {
            schema_version: 1,
            task_id: "task-1".to_string(),
            attempt: 1,
            state: TaskState::Success,
            outcome: Outcome::Success,
            failure_class: FailureClass::None,
            failure_code: None,
            benchmark_score: 1.0,
            duration_ms: 1,
            agent: None,
            evaluation: None,
            patch: None,
            usage: UsageRecord::Unknown,
            warnings: Vec::new(),
        }
    }
}
