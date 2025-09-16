# Tasks Document (TDD-driven variant)

- [ ] 1. Enumerate behavior variants & regression guards (TDD pre-step)
  - File: docs/tdd/feature_change/behavior-matrix.md
  - Define exhaustive, implementation-agnostic cases for the desired behavior change (happy path, service timeout, missing key, retries, invalid inputs)
  - Add regression guards for existing behavior (invariants, backward-compatibility checks)
  - Capture Given/When/Then examples, explicit non-goals, and open questions
  - Purpose: Establish the complete test list and “done” criteria before any implementation
  - _Leverage: docs/templates/spec_template.md, docs/tdd/_shared/invariants.md (create if missing)_
  - _Requirements: 1.1_
  - _Prompt: Role: TDD Analyst specializing in behavioral analysis | Task: Create an exhaustive behavior matrix for the requested change, listing positive, negative, edge, and failure-mode cases; add regression guards for current behavior; express each as concise, testable case names with brief Gherkin-style examples | Restrictions: Do not reference internal design; no code changes; keep wording implementation-agnostic; note unknowns as open questions | Success: Matrix covers happy path, error/timeout/missing-data scenarios, invariants, and non-goals; every item is independently testable and traceable to requirement 1.1; stakeholders agree this list defines “done”_

- [ ] 2. Red #1 — Highest-value happy path (from matrix ID BM-001)
  - File: tests/feature/bm001.happy-path.spec.ts
  - Define one end-to-end, assertion-driven test using AAA (Arrange–Act–Assert)
  - Prefer interface-level doubles only at boundaries; deterministic & implementation-agnostic
  - Purpose: Create the first “red” that drives public interface decisions
  - _Leverage: docs/tdd/feature_change/behavior-matrix.md, tests/_shared/helpers.ts (create if missing)_
  - _Requirements: 1.2_
  - _Prompt: Role: TDD Engineer focused on interface-first design | Task: Implement one automated test for BM-001 starting from explicit assertions; keep doubles at boundaries only | Restrictions: Must include meaningful assertions; do not translate the entire matrix; avoid peeking into internals | Success: Test fails for the intended reason (“red”); intent and chosen order documented in test header_

- [ ] 3. Green #1 — Smallest production change to pass BM-001
  - File: src/feature/index.ts
  - Implement only the minimal logic needed for `bm001.happy-path.spec.ts` to pass
  - Record any newly discovered cases back into the matrix
  - Purpose: Achieve a real “green” without speculative design
  - _Leverage: tests/feature/bm001.happy-path.spec.ts, docs/tdd/feature_change/behavior-matrix.md_
  - _Requirements: 1.3_
  - _Prompt: Role: Implementation-focused TDD Developer | Task: Change `src/feature/index.ts` just enough to make BM-001 pass; capture new cases in the matrix | Restrictions: Don’t weaken assertions; no premature abstractions | Success: Test passes for the intended reason; diff is minimal and readable; commit history clearly shows Red→Green_

- [ ] 4. Refactor #1 — While green, improve design safely
  - File: src/feature/index.ts (and related sourcemap if introduced)
  - Apply small, behavior-preserving refactors (naming, duplication removal, extraction) justified by BM-001 only
  - Purpose: Keep code clean as behavior stabilizes
  - _Leverage: tests/feature/bm001.happy-path.spec.ts_
  - _Requirements: 1.4_
  - _Prompt: Role: Clean-code Refactorer | Task: Refactor with tests green | Restrictions: No behavior changes; do not introduce new public surface | Success: All tests remain green; complexity and duplication trend down_

- [ ] 5. Red #2 — Error: missing key (BM-002)
  - File: tests/feature/bm002.missing-key.spec.ts
  - Add one end-to-end test that asserts correct failure mode & message/typed error
  - Purpose: Drive error-contract clarity at the public interface
  - _Leverage: docs/tdd/feature_change/behavior-matrix.md, tests/_shared/fakes.ts (create if missing)_
  - _Requirements: 1.5_
  - _Prompt: Role: TDD Engineer | Task: Implement one automated test for BM-002 | Restrictions: Deterministic; interface-level doubles only; assert on public outputs/errors | Success: Test is “red” for the intended reason and documents the contract_

- [ ] 6. Green #2 — Minimal changes for BM-002
  - File: src/feature/index.ts
  - Implement minimal logic to satisfy BM-002; update matrix with discoveries
  - Purpose: Expand behavior safely
  - _Leverage: tests/feature/bm002.missing-key.spec.ts_
  - _Requirements: 1.6_
  - _Prompt: Role: Implementation-focused TDD Developer | Task: Make BM-002 pass with smallest change | Restrictions: Don’t break BM-001; no speculative generalization | Success: Both BM-001 and BM-002 pass; diff is minimal_

- [ ] 7. Refactor #2 — Consolidate validation & error mapping
  - File: src/feature/index.ts, src/utils/validation.ts (create if missing)
  - Extract/centralize validation paths now covered by BM-001/002
  - Purpose: Reduce duplication and clarify error semantics
  - _Leverage: existing validation patterns if present_
  - _Requirements: 1.7_
  - _Prompt: Role: Clean-code Refactorer | Task: Extract shared validation with tests green | Restrictions: No behavior change; preserve error types/codes | Success: Coverage unchanged; complexity reduced; naming consistent_

- [ ] 8. Red #3 — Timeout & retry semantics (BM-003)
  - File: tests/feature/bm003.timeout-retry.spec.ts
  - Test timeout handling, backoff/limits, and observable results (status/error)
  - Purpose: Lock contract under failure/transient conditions
  - _Leverage: docs/tdd/feature_change/behavior-matrix.md, tests/_shared/clock.ts (fake timers; create if missing)_
  - _Requirements: 1.8_
  - _Prompt: Role: TDD Engineer | Task: Implement a deterministic timeout/retry test using fake timers | Restrictions: No real sleeps/network; deterministic timing | Success: Test red for intended reason; assertions describe limits/backoff clearly_

- [ ] 9. Green #3 — Implement minimal timeout/retry
  - File: src/feature/index.ts
  - Implement minimal retry/timeout per BM-003; expose only necessary knobs
  - Purpose: Fulfill contract without overfitting
  - _Leverage: tests/feature/bm003.timeout-retry.spec.ts_
  - _Requirements: 1.9_
  - _Prompt: Role: Implementation-focused TDD Developer | Task: Make BM-003 pass with smallest change | Restrictions: Do not introduce unused configuration; keep logic local | Success: BM-001..003 pass; diff minimal; public API remains small_

- [ ] 10. Contract tests for service boundary (interface-level)
  - File: tests/contract/feature.contract.spec.ts
  - Add interface-level tests for public methods covering success & failure modes learned in BM-001..003
  - Purpose: Freeze the external contract for downstream consumers
  - _Leverage: docs/tdd/feature_change/behavior-matrix.md_
  - _Requirements: 2.0_
  - _Prompt: Role: QA Engineer (Vitest) | Task: Write contract tests at the service boundary | Restrictions: No internal state inspection; assert on types/status/errors only | Success: Contract tests pass and fail appropriately when contract is violated_

- [ ] 11. Wire DI only when tests demand it
  - File: src/utils/di.ts (modify), src/services/FeatureService.ts (create if demanded)
  - Register the minimal concrete implementation required by the contract tests
  - Purpose: Introduce infrastructure strictly as pulled by tests
  - _Leverage: existing DI patterns in src/utils/di.ts_
  - _Requirements: 2.1_
  - _Prompt: Role: DevOps Engineer (IoC) | Task: Register FeatureService with correct lifetime and dependencies as required by tests | Restrictions: Avoid circular deps; keep construction minimal | Success: Tests resolve services via DI and stay green_

- [ ] 12. API slice driven by tests (optional if feature is API-facing)
  - File: src/api/feature.routes.ts, tests/api/feature.api.spec.ts
  - Create a minimal route/controller to satisfy contract/API tests; add request validation
  - Purpose: Expose behavior via HTTP without leaking internals
  - _Leverage: src/controllers/BaseController.ts, src/utils/validation.ts_
  - _Requirements: 4.2, 4.3_
  - _Prompt: Role: Full-stack Developer (Express + Vitest/Playwright) | Task: Implement smallest API surface required by tests with validation | Restrictions: Validate all inputs; correct HTTP codes; do not bypass service | Success: API tests cover CRUD/error flows; pass in CI_

- [ ] 13. Integration plan (compose components)
  - File: docs/tdd/feature_change/integration-plan.md
  - Describe integration points and test strategy (what to fake vs. hit for real)
  - Purpose: Ensure reliable integration tests
  - _Leverage: src/utils/integrationUtils.ts, tests/_shared/helpers.ts_
  - _Requirements: 6.0_
  - _Prompt: Role: Integration Engineer | Task: Plan integration approach and test layers | Restrictions: Consider all components; maintain reliability | Success: Plan reviewed; feasible; covers all integration points_

- [ ] 14. E2E tests & automation
  - File: e2e/feature.e2e.spec.ts (Playwright or Cypress), .github/workflows/ci.yml (if needed)
  - Cover critical user journeys; add deterministic fixtures and CI wiring
  - Purpose: Validate end-to-end behavior and guard regressions
  - _Leverage: tests/fixtures/data.ts, tests/_shared/helpers.ts_
  - _Requirements: All_
  - _Prompt: Role: QA Automation Engineer | Task: Implement maintainable E2E covering happy/error paths | Restrictions: Test real user workflows; avoid internal details | Success: E2E reliable in CI; covers critical journeys_

- [ ] 15. Final integration & cleanup
  - File: src/**, docs/**, tests/**
  - Integrate all components; resolve integration issues; clean up code/docs
  - Purpose: Ship with confidence
  - _Leverage: src/utils/cleanup.ts, docs/templates/_*
  - _Requirements: All_
  - _Prompt: Role: Senior Developer | Task: Perform final integration and cleanup | Restrictions: Don’t break existing functionality; meet code quality bars | Success: All tests green; docs up-to-date; system meets requirements and quality standards_
