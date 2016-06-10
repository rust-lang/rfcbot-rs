ALTER TABLE issue ADD COLUMN repository varchar(100) NOT NULL DEFAULT 'rust-lang/rust';

ALTER TABLE issuecomment ADD COLUMN repository varchar(100) NOT NULL DEFAULT 'rust-lang/rust';

ALTER TABLE milestone ADD COLUMN repository varchar(100) NOT NULL DEFAULT 'rust-lang/rust';

ALTER TABLE pullrequest ADD COLUMN repository varchar(100) NOT NULL DEFAULT 'rust-lang/rust';
