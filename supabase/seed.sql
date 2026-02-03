insert into public.users (id, email, username, age, score, is_active) values
    ('user-delete-target', 'delete@test.com', 'delete_me', 25, 100.0, true),
    ('user-alice', 'alice@test.com', 'alice', 30, 85.5, true),
    ('user-bob', 'bob@test.com', 'bob', 25, 72.0, true),
    ('user-charlie', 'charlie@test.com', 'charlie', 35, 91.2, false),
    ('user-single-target', 'single@test.com', 'single_target', 40, 50.0, true),
    ('user-update-target', 'update@test.com', 'update_me', 28, 60.0, true),
    ('user-upsert-target', 'upsert@test.com', 'upsert_me', 22, 45.0, true)
on conflict (id) do nothing;
