use logos_runtime_types::ContinuationHandle;
use logos_runtime_types::InternalEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct TurnOutcome {
    pub events: Vec<InternalEvent>,
    pub continuation: Option<ContinuationHandle>,
}
