CREATE TABLE issuelabel (
    id SERIAL PRIMARY KEY,
    fk_issue INTEGER NOT NULL REFERENCES issue (number),
    label VARCHAR NOT NULL,
    color VARCHAR NOT NULL
)
