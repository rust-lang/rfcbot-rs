CREATE TABLE build (
    id SERIAL PRIMARY KEY,
    number INTEGER NOT NULL,
    builder_name TEXT NOT NULL,
    successful BOOLEAN NOT NULL,
    message TEXT NOT NULL,
    duration_secs INTEGER,
    UNIQUE (number, builder_name)
)
