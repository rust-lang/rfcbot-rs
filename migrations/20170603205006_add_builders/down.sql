ALTER TABLE build
    DROP COLUMN builder_name,
    DROP COLUMN builder_os;
ALTER TABLE build
    RENAME COLUMN builder_env TO builder_name;
