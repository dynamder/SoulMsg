<!--
Sync Impact Report - Constitution v1.0.1
========================================

Version Change: 1.0.0 → 1.0.1 (PATCH)

Modified Principles:
- Technology Standards: Changed "nom" to "winnow" for parsing library

Templates Status:
✅ plan-template.md - Already aligned (uses winnow)
✅ spec-template.md - No changes required
✅ tasks_template.md - No changes required

Follow-up TODOs:
- None

-->

# SoulMsg Constitution

## Core Principles

### I. Code Quality (NON-NEGOTIABLE)
All code MUST meet the following quality standards: Code MUST be self-documenting with clear variable and function names; Functions MUST NOT exceed 80 lines; Files MUST NOT exceed 600 lines; Cyclomatic complexity MUST remain below 10; Code MUST follow SOLID principles; All code changes MUST pass linting and type checking before merging.

### II. User Experience First
Every feature MUST be designed with the end user in mind: User workflows MUST be intuitive and require minimal steps; Error messages MUST be clear, actionable, and user-friendly; 

### III. Testable Units (NON-NEGOTIABLE)
All production code MUST be covered by unit tests: Every function/method MUST have at least one unit test; Test coverage MUST exceed 80% for core business logic; Tests MUST be fast (< 100ms per test suite); Tests MUST be independent and not depend on execution order; Mock external dependencies; Use Arrange-Act-Assert pattern.

### IV. Good Maintainability
Codebase MUST be easy to understand, modify, and extend: Modules MUST have single responsibility; Dependencies MUST point inward (high-level modules depend on low-level modules); Configuration MUST be externalized (no hardcoded values); Documentation MUST explain "why" not just "what"; Technical debt MUST be tracked and addressed in sprint planning; Code reviews MUST verify maintainability.

### V. Simple and Concise Code Style
Complexity MUST be avoided unless absolutely necessary: Prefer explicit over implicit;  Use the simplest solution that works (YAGNI); Avoid clever code that sacrifices readability; Keep functions small and focused; Use meaningful names (no single-letter variables except loop counters); Remove dead code immediately.

### VI. MVP First
Features MUST be delivered in minimum viable increments: Start with the smallest feature set that delivers value; User stories MUST be prioritized by business value; Each increment MUST be independently testable and deployable; Scope creep MUST be actively resisted;  Post-MVP features SHOULD be marked as enhancements.

## Additional Constraints

### Technology Standards
- use clippy
- winnow for parsing
- All dependencies MUST have active maintenance and security support

### Code Style Enforcement
- Prettier for formatting

### Security Requirements
- No secrets in source code (use environment variables)
- Input validation on all user files

## Development Workflow

### Quality Gates (All MUST Pass)
1. All tests MUST pass (unit, integration)
2. Code coverage MUST meet threshold (80% core, 70% overall)
3. Linting MUST pass with no warnings
4. Type checking MUST pass with no errors
5. Security scan MUST pass (no critical/high vulnerabilities)
6. Code review MUST be approved by at least one reviewer

### Review Process
- PRs MUST have description of changes and rationale
- PRs MUST link to related issue/task
- Reviewers MUST verify: functionality, tests, style, documentation
- Commits MUST be atomic and descriptive

### Deployment
- Only tested code MAY be deployed
- Deployment artifacts MUST be reproducible
- Rollback plan MUST exist for each deployment
- Production deployments MUST be logged

## Governance

### Amendment Procedure
1. Proposed changes MUST be documented with rationale
2. Changes MUST be reviewed for impact on existing code
3. Migration plan MUST be provided if breaking changes required
4. Changes MUST be approved by project maintainer
5. Version MUST be incremented per semantic versioning rules

### Versioning Policy
- MAJOR: Backward incompatible API changes or principle redefinitions
- MINOR: New principles or materially expanded guidance
- PATCH: Clarifications, wording fixes, non-semantic refinements

### Compliance
- All team members MUST read and understand the constitution
- Constitution compliance MUST be verified in code reviews
- Violations MUST be documented with justification if accepted

**Version**: 1.0.1 | **Ratified**: 2026-03-02 | **Last Amended**: 2026-03-07
