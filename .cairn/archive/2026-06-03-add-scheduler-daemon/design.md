## Context

The proposed headless run command provides the execution primitive needed by a daemon. Scheduling adds clocks, sleeping, persistence, and concurrency policy, so it must keep time-based effects out of the pure core.

## Decisions

### 1. Schedule evaluation is pure

**Choice:** Represent schedules, last-run state, and current time as inputs to a pure evaluator that returns due-run plans.

**Rationale:** Calendar and missed-run behavior can be tested without sleeping or reading the system clock.

### 2. Daemon owns clocks and waiting

**Choice:** The daemon shell reads the clock, sleeps or polls, loads schedule state, and starts headless runs from due plans.

**Rationale:** Time and process orchestration are imperative effects.

### 3. Interfaces isolate orchestration

**Choice:** Introduce clock, schedule-store, and run-queue interfaces so the daemon can be tested with deterministic adapters.

**Rationale:** Scheduling is cross-cutting and should not couple directly to operating-system timers or the redb store.

### 4. FCIS boundary reuses headless run summaries

**Choice:** The scheduler records run outcomes from the same summary model used by headless execution.

**Rationale:** Scheduled, CLI, and TUI runs should share status semantics and history behavior.
