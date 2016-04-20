CREATE TABLE pullrequest (
    number INTEGER PRIMARY KEY,
    state VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    body VARCHAR,
    fk_assignee INTEGER REFERENCES githubuser (id),
    fk_milestone INTEGER REFERENCES milestone (id),
    locked BOOLEAN NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    closed_at TIMESTAMP,
    merged_at TIMESTAMP,
    commits INTEGER NOT NULL,
    additions INTEGER NOT NULL,
    deletions INTEGER NOT NULL,
    changed_files INTEGER NOT NULL
)
