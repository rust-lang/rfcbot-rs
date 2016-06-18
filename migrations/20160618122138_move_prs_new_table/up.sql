CREATE TABLE pullrequest_new
(
  "id" SERIAL PRIMARY KEY,
  "number" integer NOT NULL,
  state character varying NOT NULL,
  title character varying NOT NULL,
  body character varying,
  fk_assignee integer REFERENCES githubuser (id),
  fk_milestone integer REFERENCES milestone (id),
  locked boolean NOT NULL,
  created_at timestamp without time zone NOT NULL,
  updated_at timestamp without time zone NOT NULL,
  closed_at timestamp without time zone,
  merged_at timestamp without time zone,
  commits integer NOT NULL,
  additions integer NOT NULL,
  deletions integer NOT NULL,
  changed_files integer NOT NULL,
  repository character varying(100) NOT NULL
);

INSERT INTO pullrequest_new
(
    "number",
    state,
    title,
    body,
    fk_assignee,
    fk_milestone,
    locked,
    created_at,
    updated_at,
    closed_at,
    merged_at,
    commits,
    additions,
    deletions,
    changed_files,
    repository
)
SELECT * FROM pullrequest;

DROP TABLE pullrequest;

ALTER TABLE pullrequest_new RENAME TO pullrequest;
