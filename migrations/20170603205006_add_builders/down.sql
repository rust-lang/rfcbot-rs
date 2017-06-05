ALTER TABLE build
    DROP COLUMN builder_name,
    DROP COLUMN job_id,
    DROP COLUMN os;

ALTER TABLE build
    RENAME COLUMN env TO builder_name;

ALTER TABLE build
    RENAME COLUMN build_id TO "number";
ALTER TABLE build
    ALTER COLUMN "number" TYPE INTEGER USING number::integer;
