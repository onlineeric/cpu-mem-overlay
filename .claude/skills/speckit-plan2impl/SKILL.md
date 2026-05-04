---
name: "speckit-plan2impl"
description: "Execute from plan to implementation workflow."
argument-hint: "Optional guidance for the implementation phase"
compatibility: "Requires spec-kit project structure with .specify/ directory"
metadata:
  author: "github-spec-kit-bot"
  source: "templates/commands/plan2impl.md"
user-invocable: true
disable-model-invocation: false
---


## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Phases

You MUST run the below Phase one after one

### Phase 1: Plan

run skill /speckit-plan

### Phase 2: Tasks

run skill /speckit-tasks

### Phase 3: Analyze

Run skill /speckit-analyze
Then fix all issues found in the analyze phase by your best suggestions.

### Phase 4: Implement

Run skill /speckit-implement

## Key rules

Not to ask user for anything, including permission, or any questions. User expect this skill will run automatically and silently, until the whole implementation is completed.