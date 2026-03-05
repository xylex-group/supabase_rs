# Joins Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add first-class joins/nested table querying to the Rust Supabase SDK, including API surface, migrations/fixtures, tests, and documentation.

**Architecture:** Extend `QueryBuilder` with structured join select assembly (alias + join modifiers + nested columns), feed into existing PostgREST query string builder, and back the feature with seeded join tables to verify left/inner/fk joins end-to-end.

**Tech Stack:** Rust (tokio/reqwest/serde_json), PostgREST/Supabase CLI, cargo test.

---

### Task 1: Add join fixture tables and seed data

**Files:**
- Create: `supabase/migrations/20260305_add_join_fixtures.sql`
- Modify: `supabase/seed.sql`
- Test: `cargo test select_joins -- --ignored` (integration)

**Step 1: Write the failing test**

```rust
// src/tests/methods/select_joins.rs
// queries orchestral_sections with instruments nested; asserts flute inner join filters to woodwinds only
```

**Step 2: Run test to verify it fails**

Run: `supabase db reset && cargo test select_joins -- --ignored`
Expected: FAIL (missing tables/data).

**Step 3: Write minimal implementation**

```sql
-- supabase/migrations/20260305_add_join_fixtures.sql
create table orchestral_sections (...);
create table instruments (... references orchestral_sections);
create table teams (...);
create table members (... references teams, users);
```
Add matching seed rows in `supabase/seed.sql`.

**Step 4: Run test to verify it passes**

Run: `supabase db reset && cargo test select_joins -- --ignored`
Expected: PASS (data returned as expected).

**Step 5: Commit**

```bash
git add supabase/migrations/20260305_add_join_fixtures.sql supabase/seed.sql
git commit -m "chore: add join fixture tables and seed data"
```

### Task 2: Implement join selection API in QueryBuilder

**Files:**
- Modify: `src/query_builder/mod.rs`
- Modify: `src/query_builder/builder.rs`
- Create: `src/query_builder/join.rs`
- Modify: `src/query.rs`
- Test: `src/tests/methods/unit_query_build.rs`

**Step 1: Write the failing test**

```rust
// src/tests/methods/unit_query_build.rs
// assert QueryBuilder::select_with_joins builds `select=id,name,instruments!inner(id,name)`
```

**Step 2: Run test to verify it fails**

Run: `cargo test unit_query_build`
Expected: FAIL (method missing).

**Step 3: Write minimal implementation**

```rust
// src/query_builder/join.rs
pub struct JoinSpec { alias: Option<String>, relation: String, modifier: Option<JoinModifier>, columns: Vec<SelectItem> }
pub enum JoinModifier { Inner, ForeignKey(String) }
pub enum SelectItem { Column(String), Join(JoinSpec) }
```
Expose builder helpers on `QueryBuilder`:
`select_with_joins(base_columns: &[&str], joins: Vec<JoinSpec>) -> Self`
`join(...)` sugar with alias/modifier options.
Update `Query::build` to render select from structured items if present.

**Step 4: Run test to verify it passes**

Run: `cargo test unit_query_build`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/query_builder/*.rs src/query.rs
git commit -m "feat: add structured join selection to query builder"
```

### Task 3: Integration tests for joins (left, inner, m2m, fk-specific)

**Files:**
- Create: `src/tests/methods/select_joins.rs`
- Modify: `src/tests/mod.rs`
- Modify: `src/tests/base.rs`

**Step 1: Write the failing test**

```rust
// select_joins.rs
// 1) left join returns sections + empty instruments
// 2) inner join filters to flute section
// 3) m2m teams -> users returns members
// 4) explicit fk join for start_scan/end_scan style (reuse alias + fk name)
```

**Step 2: Run test to verify it fails**

Run: `supabase db reset && cargo test select_joins -- --ignored`
Expected: FAIL (join API not wired).

**Step 3: Write minimal implementation**

Add tests using new `select_with_joins` helper and assertions on JSON.

**Step 4: Run test to verify it passes**

Run: `supabase db reset && cargo test select_joins -- --ignored`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/tests/methods/select_joins.rs src/tests/base.rs src/tests/mod.rs
git commit -m "test: add integration coverage for joins and nested selects"
```

### Task 4: Documentation updates

**Files:**
- Modify: `README.md`
- Modify: `docs/RPC.md` (query examples section)
- Modify: `ARCHITECTURE.md` (query builder capabilities)

**Step 1: Write the failing test**

```markdown
# README
- Add "Joins & nested selects" section with Rust examples mirroring Supabase docs
```

**Step 2: Run test to verify it fails**

Run: `cargo test readme` (n/a) — manual doc check, expect diff exists.

**Step 3: Write minimal implementation**

Add examples demonstrating left vs inner join, alias + fk selection, many-to-many convenience using new API methods.

**Step 4: Run test to verify it passes**

Run: `cargo fmt && cargo test unit_query_build`
Expected: Docs updated, tests still pass.

**Step 5: Commit**

```bash
git add README.md docs/RPC.md ARCHITECTURE.md
git commit -m "docs: document join and nested select support"
```

### Task 5: Final verification

**Files:**
- No new files
- Test: `cargo test unit_query_build` and `cargo test select_joins -- --ignored`

**Step 1: Write the failing test**

Reuse existing suites.

**Step 2: Run test to verify it fails**

Run: `cargo test unit_query_build select_joins -- --ignored`
Expected: All pass (should have failed before).

**Step 3: Write minimal implementation**

Ensure no extra work; update changelog entry if required.

**Step 4: Run test to verify it passes**

Same command; expect PASS.

**Step 5: Commit**

```bash
git commit -am "chore: finalize joins support"
```

---

Plan complete and saved to `docs/plans/2026-03-05-joins-support.md`. Two execution options:

1. Subagent-Driven (this session) — dispatch fresh subagent per task, review between tasks.
2. Parallel Session (separate) — open new session with executing-plans, batch execution with checkpoints.

Which approach?
