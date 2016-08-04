ALTER TABLE issue DROP COLUMN IF EXISTS repository;

ALTER TABLE issuecomment DROP COLUMN IF EXISTS repository;

ALTER TABLE milestone DROP COLUMN IF EXISTS repository;

ALTER TABLE pullrequest DROP COLUMN IF EXISTS repository;
