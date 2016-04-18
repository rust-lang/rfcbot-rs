CREATE TABLE milestone (
  id INTEGER PRIMARY KEY,
  number INTEGER NOT NULL,
  open BOOLEAN NOT NULL,
  title VARCHAR NOT NULL,
  description VARCHAR,
  fk_creator INTEGER NOT NULL REFERENCES githubuser (id),
  open_issues INTEGER NOT NULL,
  closed_issues INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  closed_at TIMESTAMP,
  due_on TIMESTAMP
)
