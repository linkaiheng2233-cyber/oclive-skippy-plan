# Custom pipeline (PIPELINE_CUSTOM)

Mode: `Default`

## Step order

1. `load_recent_context`
2. `user_emotion_analyze`
3. `event_estimate`
4. `memory_rank`
5. `build_prompt`
6. `llm_generate`
7. `postprocess`
