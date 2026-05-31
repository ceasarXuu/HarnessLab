use super::{CleanupResult, CliDockerRunner, DockerCliProvider, DockerCommandRunner};
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;

impl DockerCliProvider {
    pub fn cleanup_compose_project(project: &str) -> Result<CleanupResult> {
        Self::cleanup_compose_project_with_runner(project, &CliDockerRunner)
    }

    pub fn cleanup_compose_projects_matching(token: &str) -> Result<CleanupResult> {
        Self::cleanup_compose_projects_matching_with_runner(token, &CliDockerRunner)
    }

    pub fn cleanup_compose_projects(projects: &[String]) -> Result<CleanupResult> {
        Self::cleanup_compose_projects_with_runner(projects, &CliDockerRunner)
    }

    pub fn compose_projects_matching(token: &str) -> Result<Vec<String>> {
        Ok(matching_projects(token, &CliDockerRunner)?
            .into_iter()
            .collect())
    }

    pub(super) fn cleanup_compose_projects_matching_with_runner(
        token: &str,
        runner: &impl DockerCommandRunner,
    ) -> Result<CleanupResult> {
        if token.trim().is_empty() {
            bail!("compose project match token is empty");
        }
        let mut removed = Vec::new();
        for project in matching_projects(token, runner)? {
            let cleanup = Self::cleanup_compose_project_with_runner(&project, runner)?;
            removed.extend(cleanup.removed);
        }
        Ok(CleanupResult { removed })
    }

    pub(super) fn cleanup_compose_projects_with_runner(
        projects: &[String],
        runner: &impl DockerCommandRunner,
    ) -> Result<CleanupResult> {
        let mut removed = Vec::new();
        for project in projects {
            let cleanup = Self::cleanup_compose_project_with_runner(project, runner)?;
            removed.extend(cleanup.removed);
        }
        Ok(CleanupResult { removed })
    }

    pub(super) fn cleanup_compose_project_with_runner(
        project: &str,
        runner: &impl DockerCommandRunner,
    ) -> Result<CleanupResult> {
        if project.trim().is_empty() {
            bail!("compose project is empty");
        }
        let mut removed = Vec::new();
        for container_id in
            docker_ids(runner, &Self::compose_containers_args(project), "docker ps")?
        {
            rm_force(runner, &container_id, "docker rm")?;
            removed.push(format!("container:{container_id}"));
        }
        for network_id in docker_ids(
            runner,
            &Self::compose_networks_args(project),
            "docker network ls",
        )? {
            rm_network(runner, &network_id)?;
            removed.push(format!("network:{network_id}"));
        }
        Ok(CleanupResult { removed })
    }

    pub fn compose_containers_args(project: &str) -> Vec<String> {
        vec![
            "ps".to_string(),
            "-aq".to_string(),
            "--filter".to_string(),
            format!("label=com.docker.compose.project={project}"),
        ]
    }

    pub fn compose_networks_args(project: &str) -> Vec<String> {
        vec![
            "network".to_string(),
            "ls".to_string(),
            "-q".to_string(),
            "--filter".to_string(),
            format!("label=com.docker.compose.project={project}"),
        ]
    }

    pub fn compose_container_project_labels_args() -> Vec<String> {
        vec![
            "ps".to_string(),
            "-a".to_string(),
            "--filter".to_string(),
            "label=com.docker.compose.project".to_string(),
            "--format".to_string(),
            "{{.ID}}\t{{.Label \"com.docker.compose.project\"}}".to_string(),
        ]
    }

    pub fn compose_network_project_labels_args() -> Vec<String> {
        vec![
            "network".to_string(),
            "ls".to_string(),
            "--filter".to_string(),
            "label=com.docker.compose.project".to_string(),
            "--format".to_string(),
            "{{.ID}}\t{{.Label \"com.docker.compose.project\"}}".to_string(),
        ]
    }
}

fn matching_projects(token: &str, runner: &impl DockerCommandRunner) -> Result<BTreeSet<String>> {
    let mut projects = BTreeSet::new();
    projects.extend(project_labels(
        runner,
        &DockerCliProvider::compose_container_project_labels_args(),
        "docker ps",
    )?);
    projects.extend(project_labels(
        runner,
        &DockerCliProvider::compose_network_project_labels_args(),
        "docker network ls",
    )?);
    Ok(projects
        .into_iter()
        .filter(|project| project.contains(token))
        .collect())
}

fn project_labels(
    runner: &impl DockerCommandRunner,
    args: &[String],
    context: &str,
) -> Result<Vec<String>> {
    let output = runner.output(args).with_context(|| context.to_string())?;
    if !output.success {
        bail!(
            "{context} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("{context} emitted non-utf8 output"))?;
    Ok(stdout
        .lines()
        .filter_map(|line| line.split_once('\t').map(|(_, project)| project.trim()))
        .filter(|project| !project.is_empty() && *project != "<no value>")
        .map(ToString::to_string)
        .collect())
}

fn docker_ids(
    runner: &impl DockerCommandRunner,
    args: &[String],
    context: &str,
) -> Result<Vec<String>> {
    let output = runner.output(args).with_context(|| context.to_string())?;
    if !output.success {
        bail!(
            "{context} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("{context} emitted non-utf8 output"))?;
    Ok(stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn rm_force(runner: &impl DockerCommandRunner, id: &str, context: &str) -> Result<()> {
    let output = runner.output(&["rm".to_string(), "-f".to_string(), id.to_string()])?;
    if output.success {
        Ok(())
    } else {
        bail!(
            "{context} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}

fn rm_network(runner: &impl DockerCommandRunner, id: &str) -> Result<()> {
    let output = runner.output(&["network".to_string(), "rm".to_string(), id.to_string()])?;
    if output.success {
        Ok(())
    } else {
        bail!(
            "docker network rm failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    }
}
