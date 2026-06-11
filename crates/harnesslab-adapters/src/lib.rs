pub mod fake_patch;
pub mod fake_terminal;
pub mod protocol_artifact_contract;
pub mod protocol_contract;
pub mod protocol_contract_builtins;
pub mod protocol_registry;
pub mod registry;
pub mod swe_bench_pro;
mod swe_bench_pro_artifacts;
pub mod swe_bench_pro_protocol;
pub mod terminal_bench;
pub mod terminal_bench_protocol;

pub use fake_patch::*;
pub use fake_terminal::*;
pub use protocol_artifact_contract::*;
pub use protocol_contract::*;
pub use protocol_contract_builtins::*;
pub use protocol_registry::*;
pub use registry::*;
pub use swe_bench_pro::*;
pub use swe_bench_pro_protocol::*;
pub use terminal_bench::*;
pub use terminal_bench_protocol::*;

#[cfg(test)]
mod data_boundary_contract;
#[cfg(test)]
mod data_boundary_rule_sets;
#[cfg(test)]
mod data_boundary_scan;
#[cfg(test)]
mod data_contract_tests;
#[cfg(test)]
mod protocol_contract_tests;
