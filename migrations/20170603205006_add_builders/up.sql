ALTER TABLE build
    RENAME COLUMN builder_name TO env;
ALTER TABLE build
    RENAME COLUMN "number" TO build_id;
ALTER TABLE build
    ADD COLUMN builder_name TEXT NOT NULL DEFAULT 'buildbot',
    ADD COLUMN job_id TEXT NOT NULL DEFAULT '',
    ADD COLUMN os TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE build
    ALTER COLUMN build_id TYPE VARCHAR,
    ALTER COLUMN builder_name DROP DEFAULT,
    ALTER COLUMN os DROP DEFAULT;
UPDATE build
    SET os = 'windows'
    WHERE env LIKE '%auto-win%';
UPDATE build
    SET os = 'linux'
    WHERE env LIKE '%auto-linux%';
UPDATE build
    SET os = 'osx'
    WHERE env LIKE '%auto-mac%';
