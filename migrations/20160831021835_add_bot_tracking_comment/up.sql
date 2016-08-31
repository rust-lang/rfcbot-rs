ALTER TABLE fcp_proposal ADD COLUMN fk_bot_tracking_comment INTEGER NOT NULL REFERENCES issuecomment (id) DEFAULT 242634369;
