# Rust Style Guide (TigerStyle, Rust Edition)

## The Essence Of Style

Style is design in executable form. We optimize for:

1. Safety
2. Performance
3. Developer experience

In that order. Readability is table stakes. Good style improves correctness and operations, not just
appearance.

## Why Have Style?

Consistent style reduces ambiguity, makes review faster, and catches design flaws early. We invest
in design upfront because code is cheapest to change before it ships.

## On Simplicity And Elegance

Simplicity is not "first attempt"; it is usually the result of revision. Prefer designs that are:

- bounded
- explicit
- measurable
- easy to test

When trade-offs exist, choose the solution that is easiest to reason about under failure.

## Technical Debt

Default policy: do it right the first time. We do not knowingly ship avoidable debt that undermines
safety or predictability (for example, unbounded work, hidden allocations in hot paths, or ignored
errors).

---

## Safety

### Control Flow And Bounds

- Prefer simple, explicit control flow.
- Avoid recursion in production paths unless depth is provably bounded and documented.
- Put bounds on all loops, queues, retries, and buffers.
- Long-running loops must have explicit progress and cancellation conditions.

### Types And Data Shapes

- Use explicitly sized numeric types (`u32`, `i64`, etc.) for persisted data, wire formats, and
  protocols.
- Use `usize` primarily for indexing and in-memory sizes.
- Encode invariants in types when possible (newtypes, enums, non-empty wrappers).

### Assertions And Invariants

- Assertions are for programmer errors; operating errors are handled via `Result`.
- Assert preconditions, postconditions, and key invariants.
- Prefer split assertions:
  - `assert!(a); assert!(b);`
  - not `assert!(a && b);`
- Assert both positive and negative spaces in tests (valid and invalid inputs).
- Use compile-time checks where possible:
  - const assertions (via patterns or crates where justified)
  - size/layout checks for serialized or FFI-critical structs

### Error Handling

- Never ignore `Result`.
- Use `?` for propagation.
- Avoid `unwrap()`/`expect()` outside tests, prototypes, or one-time initialization where failure is
  intentionally fatal and documented.
- Libraries should prefer typed errors (`thiserror`).
- Binaries may use context-rich error aggregation (`anyhow`) at boundaries.

### Memory And Resource Discipline

- In latency-sensitive paths, avoid runtime allocation churn.
- Pre-allocate (`with_capacity`, pools, arenas) where allocation patterns are known.
- Avoid unnecessary cloning; borrow first.
- Minimize variable scope and live mutable state.

### Function Shape

- Hard limit: 70 lines per function (excluding signatures/doc comments where practical).
- Keep branching logic centralized in parent functions.
- Push pure transformations into leaf helpers.
- Prefer "push ifs up, push fors down" when splitting complex logic.

### External Event Handling

- Do not couple internal state transitions directly to external event timing.
- Prefer internal pacing, bounded work units, and batching.

---

## Performance

- Consider performance at design time, not only after profiling.
- Do back-of-the-envelope sketches for network, disk, memory, and CPU (latency + bandwidth).
- Optimize the slowest and most frequently used bottlenecks first.
- Separate control plane and data plane.
- Batch operations to amortize fixed costs.
- Be explicit in hot code:
  - keep hot loops small
  - reduce indirection
  - avoid hidden allocations
  - avoid repeated parsing/conversion in loops

---

## Developer Experience

### Naming

- Functions, variables, modules, files: `snake_case`
- Types, traits, enums: `UpperCamelCase`
- Constants/statics: `SCREAMING_SNAKE_CASE`
- No unclear abbreviations unless domain-standard (`id`, `ttl`, `cpu` are fine).
- Prefer names with units/qualifiers at the end:
  - `latency_ms_max` over `max_latency_ms`
- Name by domain intent, not implementation detail.

### Ordering

- Files are read top-down: keep high-level entrypoints near the top.
- In impl blocks, prefer:
  1. constructors
  2. public methods
  3. private helpers
- Keep related types and logic close; extract only when it improves cohesion.

### Comments And Docs

- Always explain why, not just what.
- Public APIs require `///` docs.
- Add examples for non-obvious behavior.
- Complex tests should start with a brief goal/method comment.

### API Clarity

- Prefer explicit configuration over hidden defaults.
- Use typed option structs/builders for multi-parameter configuration.
- Avoid boolean parameter traps; use enums for mode flags.

---

## State, Ownership, And Invalidation

- Avoid duplicate mutable state and alias-heavy designs.
- Keep ownership clear at API boundaries.
- Prefer immutable data by default; mutate in narrow scopes.
- Compute/check values near use sites to reduce check-use drift.
- For large structs in critical code, prefer in-place mutation via `&mut` over needless
  copy-construct-return cycles.

---

## Off-By-One And Numeric Hygiene

- Be explicit with index/count/size semantics.
- Use checked/saturating arithmetic where overflow is plausible.
- Prefer explicit division semantics (`div_ceil`, clear floor/round behavior).
- Include units in names for time/size/rate fields.

---

## Style By The Numbers

- Line length hard limit: 100 columns.
- 4-space indentation.
- Use braces for multi-line `if`/`else`/`match` arms.
- Prefer trailing commas to improve diffs and formatter output.

Required checks:

- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`

For unsafe/concurrency-heavy code, add targeted tools where applicable:

- `cargo miri test`
- sanitizers (`asan`, `tsan`) in CI variants

---

## Dependencies And Tooling

- Prefer fewer dependencies, especially in core paths.
- Every dependency must have a clear reason and maintenance plan.
- Prefer stable, widely used crates over novelty.
- Keep the default toolbox small: `cargo`, `rustfmt`, `clippy`, `rustdoc`, tests, benches.

---

## Commit And Review Discipline

- Write commit messages that explain intent and impact.
- In code and PR notes, always state why.
- Review for safety/performance regressions first, style second.

## Final Rule

Small, explicit, bounded, testable code wins. If a rule conflicts with clarity or safety, choose
clarity and safety, then update this guide.
