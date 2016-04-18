CREATE TABLE issuecomment (
    id INTEGER PRIMARY KEY,
    fk_issue INTEGER NOT NULL REFERENCES issue (number),
    fk_user INTEGER NOT NULL REFERENCES githubuser (id),
    body VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
)
