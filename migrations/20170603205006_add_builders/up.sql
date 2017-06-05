ALTER TABLE build
    RENAME COLUMN builder_name TO env;

ALTER TABLE build
    ADD COLUMN builder_name TEXT NOT NULL DEFAULT 'buildbot',
    ADD COLUMN os TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE build
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
