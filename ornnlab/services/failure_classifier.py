from __future__ import annotations

from ornnlab.services.container_proxy_runtime import ProxyConfigurationError


def classify_exception(exc: BaseException) -> dict[str, str]:
    if isinstance(exc, ProxyConfigurationError):
        return {
            "failure_class": "proxy_configuration_failure",
            "failure_code": "docker_proxy_unavailable",
            "failure_summary": str(exc),
        }
    message = str(exc).lower()
    if "docker" in message:
        return {
            "failure_class": "docker_resource_failure",
            "failure_code": "docker_execution_failed",
            "failure_summary": str(exc),
        }
    if "dataset" in message or "benchmark" in message:
        return {
            "failure_class": "dataset_unavailable",
            "failure_code": "dataset_resolution_failed",
            "failure_summary": str(exc),
        }
    return {
        "failure_class": "harbor_internal_error",
        "failure_code": exc.__class__.__name__,
        "failure_summary": str(exc),
    }
