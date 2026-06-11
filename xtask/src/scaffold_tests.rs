#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn scaffold_deterministic_sample_contains_protocol_structures() {
        let dir = TempDir::new().unwrap();
        crate::scaffold::scaffold_adapter(
            "deterministic-sample",
            "harnesslab.deterministic-sample.runtime",
            dir.path(),
        )
        .unwrap();

        let module_path = dir.path().join("deterministic_sample.rs");
        let content = fs::read_to_string(&module_path).unwrap();

        assert!(
            content.contains("pub struct DeterministicSampleAdapter"),
            "generated module must declare DeterministicSampleAdapter"
        );
        assert!(
            content.contains("fn protocol_descriptor"),
            "generated module must expose protocol_descriptor"
        );
        assert!(
            content.contains("fn data_lifecycle"),
            "generated module must declare data_lifecycle"
        );
        assert!(
            content.contains("fn runtime_lifecycle"),
            "generated module must declare runtime_lifecycle"
        );
        assert!(
            content.contains("fn artifacts"),
            "generated module must declare artifacts"
        );
        assert!(
            content.contains("fn readiness_probes"),
            "generated module must declare readiness_probes"
        );
        assert!(
            content.contains("fn failure_mappings"),
            "generated module must declare failure_mappings"
        );
        assert!(
            content.contains("fn report_metadata"),
            "generated module must declare report_metadata"
        );
        assert!(
            content.contains("built_in_protocol_registry()"),
            "generated module must reference built_in_protocol_registry"
        );
    }

    #[test]
    fn scaffold_outputs_registry_binding_snippet() {
        let dir = TempDir::new().unwrap();
        crate::scaffold::scaffold_adapter(
            "my-benchmark",
            "harnesslab.my-benchmark.runtime",
            dir.path(),
        )
        .unwrap();

        let binding_path = dir.path().join("registry_binding.rs");
        let content = fs::read_to_string(&binding_path).unwrap();

        assert!(
            content.contains("binding("),
            "binding snippet must contain binding macro call"
        );
        assert!(
            content.contains("\"my-benchmark\""),
            "binding snippet must reference benchmark id"
        );
        assert!(
            content.contains("\"harnesslab.my-benchmark.runtime\""),
            "binding snippet must reference adapter id"
        );
        assert!(
            content.contains("AdapterStability::Experimental"),
            "binding snippet must set Experimental stability"
        );
    }
}
