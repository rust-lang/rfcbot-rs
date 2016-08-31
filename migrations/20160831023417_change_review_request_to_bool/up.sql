ALTER TABLE fcp_review_request DROP COLUMN fk_reviewed_comment;
ALTER TABLE fcp_review_request ADD COLUMN reviewed BOOLEAN NOT NULL;
