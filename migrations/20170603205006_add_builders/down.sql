ALTER TABLE build
    DROP COLUMN builder_name,
    DROP COLUMN os;
ALTER TABLE build
    RENAME COLUMN env TO builder_name;
