<!--
SYNC IMPACT REPORT
==================
Version change: (template / unratified) → 1.0.0
Bump rationale: Initial ratification — first concrete constitution replacing the
unmodified Spec Kit placeholder template. MAJOR version (1.0.0) is appropriate
for a from-zero adoption of governance.

Modified principles:
- [PRINCIPLE_1_NAME] → I. Rust Best Practices via rust-skills
- [PRINCIPLE_2_NAME] → II. Comprehensive Unit Test Coverage (NON-NEGOTIABLE)
- [PRINCIPLE_3_NAME] → III. Post-Implementation Self-Review
- [PRINCIPLE_4_NAME] → IV. Green Tests Before Done (NON-NEGOTIABLE)
- [PRINCIPLE_5_NAME] → (intentionally removed; user requested four principles)

Added sections:
- Code Quality Standards
- Development Workflow & Quality Gates
- Governance (concrete content)

Removed sections:
- None (placeholder Section 5 / Principle 5 collapsed; user specified four principles)

Templates requiring updates:
- ✅ .specify/templates/plan-template.md — Constitution Check gates encoded against
  Principles I–IV.
- ✅ .specify/templates/tasks-template.md — Unit-test tasks promoted from OPTIONAL
  to MANDATORY per Principle II; review/test-green tasks added per III & IV.
- ✅ .specify/templates/spec-template.md — No structural change required;
  technology-agnostic spec content remains compatible.

Follow-up TODOs:
- None.
-->

# CPU/Memory Overlay Constitution

## Core Principles

### I. Rust Best Practices via rust-skills

All Rust code in this project MUST be written, reviewed, and refactored against
the guidance in the `rust-skills` skill (179 rules across 14 categories: ownership,
error handling, async patterns, API design, memory optimization, performance,
testing, anti-patterns, and more). Before writing or modifying Rust code, the
implementer MUST consult `rust-skills` for the relevant category and apply the
listed rules. Code that violates a `rust-skills` rule MUST be flagged in review
and either corrected or accompanied by a written justification recorded in the
PR description.

**Rationale**: A single, authoritative source for Rust idioms eliminates
bikeshedding, prevents the slow drift toward non-idiomatic patterns, and ensures
that reviewers and implementers evaluate code against the same baseline.

### II. Comprehensive Unit Test Coverage (NON-NEGOTIABLE)

Every module that contains non-trivial logic MUST ship with unit tests. "Enough
coverage" is defined as: every public function, every distinct branch in pure
logic (formatting, parsing, fallback handling, error paths), and every
documented edge case from the feature spec MUST have at least one assertion.
Trivial getters, `Default` impls, and pure delegations are exempt. UI rendering
behavior that cannot be exercised without a windowing system MAY be covered by
manual smoke tests recorded in the feature's quickstart, but the underlying
data-shaping logic MUST be unit-tested.

**Rationale**: Unit tests are the cheapest, fastest feedback loop. Skipping them
to "save time" trades a few minutes today for hours of regression debugging
later. A test that pins down branch behavior survives refactors; a test that
only exercises the happy path does not.

### III. Post-Implementation Self-Review

After completing a coding task and before declaring it done, the implementer
MUST re-read the diff and reflect on it against the project's coding preferences
(clean naming, single responsibility, DRY, explicit over implicit, purpose-built
libraries over manual implementations) and `rust-skills` rules. If the review
surfaces a violation or a clearer alternative, the code MUST be refactored
before the task is marked complete. The implementer MUST NOT defer this review
to a later PR or to the reviewer.

**Rationale**: First-draft code reflects the path the author took to make it
work, not the path a reader will take to understand it. A self-review pass
catches the obvious cleanups (dead code, leftover scaffolding, inconsistent
naming, unnecessary abstractions) before they enter the review queue and waste
a reviewer's time.

### IV. Green Tests Before Done (NON-NEGOTIABLE)

After completing coding and self-review, the implementer MUST run the full unit
test suite (`cargo test` for Rust crates) and confirm that every test passes.
A task with failing tests is not complete — regardless of whether the failure
is in newly written code, pre-existing tests, or tests for unrelated modules.
If a test fails, the implementer MUST diagnose the root cause and fix it (or
update the test if the behavior change was intentional and approved) before
declaring the task done. Disabling, skipping, or `#[ignore]`-ing a test to make
the suite green is forbidden without an explicit, written justification linked
to a follow-up task.

**Rationale**: A red suite that the team has learned to ignore is worse than
no suite at all — it trains everyone to dismiss real failures. Holding the
"done" bar at green protects the meaning of the test signal.

## Code Quality Standards

The following standards apply to all code in this repository in addition to the
Core Principles above:

- **Readability first**: Code MUST be self-documenting via clear naming. Comments
  are reserved for non-obvious *why* (hidden constraints, subtle invariants,
  workarounds for specific bugs). Comments that restate *what* the code does
  are forbidden.
- **Small, focused units**: Functions, types, and files MUST follow the single
  responsibility principle. When a function exceeds one screen or a file
  exceeds a few hundred lines without a clear reason, it MUST be split.
- **DRY with judgment**: Reusable logic MUST be extracted into shared functions
  or modules. However, three similar lines are better than a premature
  abstraction; extraction MUST be driven by a real second use case, not a
  hypothetical future one.
- **Purpose-built libraries**: For structured data (HTML, JSON, XML, dates,
  URLs), the implementer MUST use a battle-tested library (`cheerio`,
  `serde_json`, `chrono`/`time`, `url`, etc.) rather than hand-rolled regex or
  string manipulation. New crate dependencies MUST favor active maintenance,
  a readable API, and (for Rust) idiomatic API design per `rust-skills`.
- **No backwards-compatibility cruft**: Removed code MUST be deleted, not
  commented out or shimmed. Renamed-but-still-exported aliases are forbidden
  unless an external consumer is documented.

## Development Workflow & Quality Gates

The following gates MUST be satisfied for every change before it is merged or
declared complete:

1. **Plan gate** (for features going through Spec Kit): The implementation plan's
   Constitution Check section MUST explicitly evaluate compliance with
   Principles I–IV and record any deviation in the Complexity Tracking table
   with a written justification.
2. **Implementation gate**: Each implementation task that produces or modifies
   Rust code MUST be paired with the corresponding unit-test task in the same
   user-story phase. Implementation tasks MUST NOT be marked complete before
   their paired tests are written and passing.
3. **Self-review gate** (Principle III): Before marking the final implementation
   task of a feature complete, the implementer MUST add an explicit self-review
   step (or task) confirming the diff has been re-read and refactored.
4. **Test-green gate** (Principle IV): Before marking the feature complete,
   `cargo test` (or the equivalent test command for non-Rust additions) MUST
   pass with zero failures and zero unjustified `#[ignore]` additions.

Reviewers MUST verify these gates were honored before approving. A PR that
violates a gate MUST be sent back for revision regardless of how small the
change appears.

## Governance

This constitution supersedes ad-hoc team conventions, individual preferences,
and prior informal practices wherever they conflict.

**Amendment procedure**: Amendments MUST be proposed via a PR that updates this
file, increments the version per the policy below, sets `LAST_AMENDED_DATE` to
the merge date, and updates the Sync Impact Report at the top of the file. The
PR MUST also update any dependent templates (`.specify/templates/*.md`) and any
runtime guidance docs (`README.md`, `CLAUDE.md`, agent guidance files) whose
content depends on the changed principles. Amendments require approval from
the project owner.

**Versioning policy** (semantic):

- **MAJOR**: A principle is removed, redefined in a backward-incompatible way,
  or the governance procedure itself changes incompatibly.
- **MINOR**: A new principle or section is added, or existing guidance is
  materially expanded.
- **PATCH**: Clarifications, wording fixes, typo corrections, or non-semantic
  refinements.

**Compliance review**: At the start of every Spec Kit `/speckit-plan` run, the
plan's Constitution Check section MUST evaluate the feature against the current
version of this file. Any violation MUST be either eliminated from the plan or
recorded in the Complexity Tracking table with a written justification. At
review time, the reviewer MUST confirm the Constitution Check is present and
honest before approving.

**Runtime guidance**: This constitution governs *what* must be true. For *how*
to apply the principles in day-to-day code, consult `rust-skills` (Rust),
`CLAUDE.md` (general coding preferences), and the active feature plan in
`specs/<feature>/plan.md`.

**Version**: 1.0.0 | **Ratified**: 2026-05-04 | **Last Amended**: 2026-05-04
