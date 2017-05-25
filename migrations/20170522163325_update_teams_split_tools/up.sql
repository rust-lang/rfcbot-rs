-- create new teams
INSERT INTO teams (name, ping, label) VALUES
  ('Infrastructure', 'rust-lang/infra', 'T-infra'),
  ('Cargo', 'rust-lang/cargo', 'T-cargo'),
  ('Dev tools', 'rust-lang/dev-tools', 'T-dev-tools');

-- create infra, cargo, dev-tools memberships
INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE
  (
    t.ping = 'rust-lang/dev-tools' AND
    u.login = ANY(ARRAY['japaric', 'brson', 'nrc', 'michaelwoerister', 'killercup', 'jonathandturner'])
  ) OR
  (
    t.ping = 'rust-lang/cargo' AND
    u.login = ANY(ARRAY['alexcrichton', 'carols10cents', 'withoutboats', 'matklad', 'wycats', 'aturon'])
  ) OR
  (
    t.ping = 'rust-lang/infra' AND
    u.login = ANY(ARRAY['brson', 'alexcrichton', 'frewsxcv', 'shepmaster', 'aidanhs', 'TimNN', 'carols10cents', 'Mark-Simulacrum', 'erickt', 'aturon'])
  );

-- remove peschkaj from docs team
DELETE FROM memberships m USING githubuser u, teams t
WHERE
    m.fk_member = u.id AND
    m.fk_team = t.id AND
    u.login = 'peschkaj' AND t.ping = 'rust-lang/docs';
