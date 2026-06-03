#[derive(Debug, Clone, Copy)]
pub(super) enum ExecutionMode {
    New,
    Resume,
    Replay,
}
