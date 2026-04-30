# Recorded Runs

Put manual LLM benchmark result JSON files in this directory.

Each record should include:

```json
{
  "task_id": "api.todo_crud",
  "target": "hayulo",
  "model": "codex",
  "status": "passed",
  "metrics": {
    "repair_iterations": 0
  }
}
```
