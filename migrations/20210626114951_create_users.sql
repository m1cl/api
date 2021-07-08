-- Add migration script here
CREATE TABLE users(
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    username varchar NOT NULL UNIQUE,
    password varchar NOT NULL,
    created_at TIMESTAMPTZ not null Default 'NOW'::timestamptz
);

