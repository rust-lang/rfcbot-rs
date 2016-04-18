CREATE TABLE issue (
  number INTEGER PRIMARY KEY,
  fk_milestone INTEGER REFERENCES milestone (id),
  fk_user INTEGER NOT NULL REFERENCES githubuser (id),
  fk_assignee INTEGER REFERENCES githubuser (id),
  open BOOLEAN NOT NULL,
  is_pull_request BOOLEAN NOT NULL,
  title VARCHAR NOT NULL,
  body VARCHAR NOT NULL,
  locked BOOLEAN NOT NULL,
  closed_at TIMESTAMP,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
)
