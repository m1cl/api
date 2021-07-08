-- Add migration script here
CREATE TABLE media
(name text PRIMARY KEY,
owner text references users(username) ON DELETE set null ON UPDATE CASCADE,
type text NOT NULL,
data bytea NOT NULL);
