-- RPC Test Functions Setup
-- This file creates test functions for RPC testing

-- Clean up any existing test functions
DROP FUNCTION IF EXISTS test_void_func CASCADE;
DROP FUNCTION IF EXISTS test_echo CASCADE;
DROP FUNCTION IF EXISTS test_get_test_rows CASCADE;
DROP FUNCTION IF EXISTS test_add_numbers CASCADE;
DROP FUNCTION IF EXISTS test_json_echo CASCADE;

-- Void return function
CREATE OR REPLACE FUNCTION test_void_func() RETURNS void AS $$
BEGIN
    -- do nothing, just return void
    NULL;
END;
$$ LANGUAGE plpgsql;

-- Scalar return function with text parameter
CREATE OR REPLACE FUNCTION test_echo(val text) RETURNS text AS $$
BEGIN
    RETURN val;
END;
$$ LANGUAGE plpgsql;

-- Set return function (returns rows from test table)
-- Note: This assumes there's a 'test' table with columns 'id' and 'dog'
CREATE OR REPLACE FUNCTION test_get_test_rows() RETURNS SETOF test AS $$
BEGIN
    RETURN QUERY SELECT * FROM test;
END;
$$ LANGUAGE plpgsql;

-- Function with multiple parameter types
CREATE OR REPLACE FUNCTION test_add_numbers(a integer, b integer) RETURNS integer AS $$
BEGIN
    RETURN a + b;
END;
$$ LANGUAGE plpgsql;

-- Function with JSON parameter
CREATE OR REPLACE FUNCTION test_json_echo(data jsonb) RETURNS jsonb AS $$
BEGIN
    RETURN data;
END;
$$ LANGUAGE plpgsql;

-- Function with default parameter
CREATE OR REPLACE FUNCTION test_greet(name text DEFAULT 'World') RETURNS text AS $$
BEGIN
    RETURN 'Hello, ' || name || '!';
END;
$$ LANGUAGE plpgsql;