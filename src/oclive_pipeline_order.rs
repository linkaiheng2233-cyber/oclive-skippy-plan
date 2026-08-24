//! 编排步骤顺序快照（`oclive init --pipeline Default`）。
//! 完整宿主以 oclivenewnew `process_message` 为准。

pub const OCLIVE_PIPELINE_STEPS: &[&str] = &[
    "load_recent_context",
    "user_emotion_analyze",
    "event_estimate",
    "memory_rank",
    "build_prompt",
    "llm_generate",
    "postprocess",
];
