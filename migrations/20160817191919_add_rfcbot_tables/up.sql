CREATE TABLE fcp_proposal (
    id SERIAL PRIMARY KEY,
    fk_issue INTEGER NOT NULL REFERENCES issue (id),
    fk_initiator INTEGER NOT NULL REFERENCES githubuser (id),
    fk_initiating_comment INTEGER NOT NULL REFERENCES issuecomment (id),
    disposition VARCHAR NOT NULL
);

CREATE TABLE fcp_review_request (
    id SERIAL PRIMARY KEY,
    fk_proposal INTEGER NOT NULL REFERENCES fcp_proposal (id) ON DELETE CASCADE,
    fk_reviewer INTEGER NOT NULL REFERENCES githubuser (id),
    fk_reviewed_comment INTEGER REFERENCES issuecomment (id)
);

CREATE TABLE fcp_concern (
    id SERIAL PRIMARY KEY,
    fk_proposal INTEGER NOT NULL REFERENCES fcp_proposal (id) ON DELETE CASCADE,
    fk_initiator INTEGER NOT NULL REFERENCES githubuser (id),
    fk_resolved_comment INTEGER REFERENCES issuecomment (id),
    name VARCHAR NOT NULL
);

CREATE TABLE rfc_feedback_request (
    id SERIAL PRIMARY KEY,
    fk_initiator INTEGER NOT NULL REFERENCES githubuser (id),
    fk_requested INTEGER NOT NULL REFERENCES githubuser (id),
    fk_feedback_comment INTEGER REFERENCES issuecomment (id)
);
