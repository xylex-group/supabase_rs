create table if not exists public.users (
    id text primary key default gen_random_uuid()::text,
    email text,
    username text unique,
    age integer,
    score numeric,
    is_active boolean default true,
    metadata jsonb,
    created_at timestamptz default now(),
    updated_at timestamptz default now()
);

-- RLS permissive for testing
alter table public.users enable row level security;

create policy "Allow all on users" on public.users
    for all using (true) with check (true);
