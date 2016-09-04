ALTER TABLE fcp_review_request DROP COLUMN reviewed;
ALTER TABLE fcp_review_request ADD COLUMN fk_reviewed_comment INTEGER REFERENCES issuecomment (id);
