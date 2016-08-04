CREATE TABLE issue_new
(
    "id" SERIAL PRIMARY KEY,
    "number" integer NOT NULL,
    fk_milestone integer REFERENCES milestone (id),
    fk_user integer NOT NULL REFERENCES githubuser (id),
    fk_assignee integer REFERENCES githubuser (id),
    open boolean NOT NULL,
    is_pull_request boolean NOT NULL,
    title character varying NOT NULL,
    body character varying NOT NULL,
    locked boolean NOT NULL,
    closed_at timestamp without time zone,
    created_at timestamp without time zone NOT NULL,
    updated_at timestamp without time zone NOT NULL,
    labels text[] NOT NULL,
    repository character varying(100) NOT NULL
);

INSERT INTO issue_new
(
    "number",
    fk_milestone,
    fk_user,
    fk_assignee,
    open,
    is_pull_request,
    title,
    body,
    locked,
    closed_at,
    created_at,
    updated_at,
    labels,
    repository
)
SELECT * FROM issue;

ALTER TABLE issuecomment DROP CONSTRAINT issuecomment_fk_issue_fkey;

DROP TABLE issue;

ALTER TABLE issue_new RENAME TO issue;

UPDATE issuecomment ic SET fk_issue = (SELECT i.id FROM issue i WHERE i.number = ic.fk_issue);

ALTER TABLE issuecomment
ADD CONSTRAINT issuecomment_fk_issue_fkey
FOREIGN KEY (fk_issue)
REFERENCES issue (id);
