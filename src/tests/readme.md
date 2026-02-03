This directory contains integration tests that require a live Supabase instance.

## Local Setup (Recommended)

Install [Supabase CLI](https://github.com/supabase/cli#install-the-cli) and Docker, then:

```bash
supabase start
```

Create `.env` in project root:
```env
SUPABASE_URL=http://127.0.0.1:54321
SUPABASE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0
```

Run tests:
```bash
supabase db reset && cargo test
```

> **Note:** `db reset` re-seeds the database. Some tests consume seeded rows.

## Remote Setup

Create `.env` in project root:
```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_KEY=your-anon-key
```

Run tests:
```bash
cargo test
```

## Unit Tests Only

```bash
cargo test unit_
```

## Adding Tests

- Add `METHOD_NAME.rs` in `/methods`
- Add to `mod.rs`
- Import in `base.rs`
- Use `#[tokio::test]` macro
