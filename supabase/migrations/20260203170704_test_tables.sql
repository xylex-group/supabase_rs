-- Test tables for supabase_rs integration tests
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

create table if not exists public.posts (
    id text primary key default gen_random_uuid()::text,
    user_id text references public.users(id) on delete cascade,
    title text not null,
    content text,
    view_count integer default 0,
    published boolean default false,
    created_at timestamptz default now()
);

-- RLS permissive for testing
alter table public.users enable row level security;
alter table public.posts enable row level security;

create policy "Allow all on users" on public.users
    for all using (true) with check (true);

create policy "Allow all on posts" on public.posts
    for all using (true) with check (true);
