use crate::{
    BenchmarkAdapter, DeterministicSampleAdapter, ProtocolAdapterDescriptor,
    SweBenchProAdapter, TerminalBenchAdapter,
};

pub fn built_in_protocol_adapter_descriptors() -> Vec<ProtocolAdapterDescriptor> {
    vec![
        TerminalBenchAdapter::new()
            .protocol_descriptor()
            .expect("terminal-bench adapter must expose a protocol descriptor"),
        SweBenchProAdapter::new()
            .protocol_descriptor()
            .expect("swe-bench-pro adapter must expose a protocol descriptor"),
        DeterministicSampleAdapter
            .protocol_descriptor()
            .expect("deterministic-sample adapter must expose a protocol descriptor"),
    ]
}
