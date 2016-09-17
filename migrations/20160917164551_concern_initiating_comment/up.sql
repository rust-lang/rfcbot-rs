ALTER TABLE fcp_concern ADD COLUMN fk_initiating_comment INTEGER
  NOT NULL REFERENCES issuecomment (id) DEFAULT 24899824;

ALTER TABLE fcp_concern ALTER COLUMN fk_initiating_comment DROP DEFAULT;
