insert into public.users (id, email, username, age, score, is_active) values
    ('user-delete-target', 'delete@test.com', 'delete_me', 25, 100.0, true),
    ('user-alice', 'alice@test.com', 'alice', 30, 85.5, true),
    ('user-bob', 'bob@test.com', 'bob', 25, 72.0, true),
    ('user-charlie', 'charlie@test.com', 'charlie', 35, 91.2, false),
    ('user-single-target', 'single@test.com', 'single_target', 40, 50.0, true),
    ('user-update-target', 'update@test.com', 'update_me', 28, 60.0, true),
    ('user-upsert-target', 'upsert@test.com', 'upsert_me', 22, 45.0, true)
on conflict (id) do nothing;

-- Join fixture data
insert into public.orchestral_sections (id, name) values
    ('section-woodwinds', 'woodwinds'),
    ('section-strings', 'strings'),
    ('section-empty', 'empty')
on conflict (id) do nothing;

insert into public.instruments (id, name, section_id) values
    ('inst-flute', 'flute', 'section-woodwinds'),
    ('inst-clarinet', 'clarinet', 'section-woodwinds'),
    ('inst-violin', 'violin', 'section-strings')
on conflict (id) do nothing;

insert into public.teams (id, name) values
    ('team-alpha', 'Alpha'),
    ('team-beta', 'Beta')
on conflict (id) do nothing;

insert into public.members (team_id, user_id) values
    ('team-alpha', 'user-alice'),
    ('team-alpha', 'user-bob'),
    ('team-beta', 'user-charlie')
on conflict (team_id, user_id) do nothing;
