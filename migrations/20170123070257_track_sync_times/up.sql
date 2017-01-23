CREATE TABLE githubsync (
    id SERIAL PRIMARY KEY,
    successful BOOLEAN NOT NULL,
    ran_at TIMESTAMP NOT NULL,
    message VARCHAR
);

INSERT INTO githubsync (successful, ran_at, message) VALUES
  (true, now() - interval '2 hours', null);
