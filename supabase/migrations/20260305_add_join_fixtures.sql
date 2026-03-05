-- Join fixture tables for testing nested selects and resource embedding

-- Orchestral sections (parent)
create table if not exists public.orchestral_sections (
    id text primary key default gen_random_uuid()::text,
    name text not null
);

-- Instruments (child, many-to-one to orchestral_sections)
create table if not exists public.instruments (
    id text primary key default gen_random_uuid()::text,
    name text not null,
    section_id text not null references public.orchestral_sections(id) on delete cascade
);

-- Teams
create table if not exists public.teams (
    id text primary key default gen_random_uuid()::text,
    name text not null
);

-- Members: junction table for teams <-> users (many-to-many)
create table if not exists public.members (
    team_id text not null references public.teams(id) on delete cascade,
    user_id text not null references public.users(id) on delete cascade,
    primary key (team_id, user_id)
);

-- RLS permissive for testing
alter table public.orchestral_sections enable row level security;
alter table public.instruments enable row level security;
alter table public.teams enable row level security;
alter table public.members enable row level security;

create policy "Allow all on orchestral_sections" on public.orchestral_sections
    for all using (true) with check (true);

create policy "Allow all on instruments" on public.instruments
    for all using (true) with check (true);

create policy "Allow all on teams" on public.teams
    for all using (true) with check (true);

create policy "Allow all on members" on public.members
    for all using (true) with check (true);
