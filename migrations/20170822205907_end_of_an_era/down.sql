INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE u.login = 'brson' AND (
  t.ping = 'rust-lang/core' OR
  t.ping = 'rust-lang/libs' OR
  t.ping = 'rust-lang/tools' OR
  t.ping = 'rust-lang/infra' OR
  t.ping = 'rust-lang/dev-tools'
);
