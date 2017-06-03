ALTER TABLE build
    RENAME COLUMN builder_name TO builder_env;

ALTER TABLE build
    ADD COLUMN builder_name TEXT NOT NULL DEFAULT 'buildbot',
    ADD COLUMN builder_os TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE build
    ALTER COLUMN builder_name DROP DEFAULT,
    ALTER COLUMN builder_os DROP DEFAULT;
UPDATE build
    SET builder_os = 'windows'
    WHERE builder_env LIKE '%auto-win%';
UPDATE build
    SET builder_os = 'linux'
    WHERE builder_env LIKE '%auto-linux%';
UPDATE build
    SET builder_os = 'osx'
    WHERE builder_env LIKE '%auto-mac%';
