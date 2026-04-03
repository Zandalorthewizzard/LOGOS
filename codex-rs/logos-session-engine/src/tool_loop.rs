use logos_runtime_types::InternalEvent;
use logos_runtime_types::InternalMessage;
use logos_tool_runtime::PendingToolCall;
use logos_tool_runtime::ToolCallCollector;

pub fn tool_messages_from_events(events: &[InternalEvent]) -> Vec<InternalMessage> {
    let mut collector = ToolCallCollector::default();
    for event in events {
        collector.observe(event);
    }
    collector.into_messages()
}

pub fn tool_calls_from_events(events: &[InternalEvent]) -> Vec<PendingToolCall> {
    let mut collector = ToolCallCollector::default();
    for event in events {
        collector.observe(event);
    }
    collector.into_calls()
}
