This directory contains integration-like tests that require a live Supabase project.

## Local Setup (Recommended)

1. Install [Supabase CLI](https://github.com/supabase/cli#install-the-cli) and Docker
2. Start local instance:
   ```bash
   supabase start
   supabase db reset
   ```
3. Create `.env` in project root:
   ```env
   SUPABASE_URL=http://127.0.0.1:54321
   SUPABASE_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0
   ```
4. Run tests:
   ```bash
   supabase db reset && cargo test
   ```

> **Note:** `db reset` re-seeds the database. Some tests (e.g. delete) consume seeded rows.

## Remote Setup

Use your Supabase project credentials in `.env`:
```env
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_KEY=your-anon-key
```

## Unit Tests Only

No network required:
```bash
cargo test unit_
```

## Adding Tests

- Add `METHOD_NAME.rs` in `/methods`
- Add to `mod.rs`
- Import in `base.rs`
- Run under `#[tokio::test]` macro
