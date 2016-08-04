CREATE TABLE issue_old
(
    "number" integer NOT NULL PRIMARY KEY,
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

INSERT INTO issue_old
SELECT
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
FROM issue;

ALTER TABLE issuecomment DROP CONSTRAINT issuecomment_fk_issue_fkey;

UPDATE issuecomment ic SET fk_issue = (SELECT issue.number FROM issue WHERE issue.id = ic.fk_issue);

DROP TABLE issue;

ALTER TABLE issue_old RENAME TO issue;

ALTER TABLE issuecomment
ADD CONSTRAINT issuecomment_fk_issue_fkey
FOREIGN KEY (fk_issue)
REFERENCES issue (number);
