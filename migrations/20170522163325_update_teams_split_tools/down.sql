DELETE FROM memberships m USING teams t
WHERE
    m.fk_team = t.id AND
    (
      t.ping = 'rust-lang/cargo' OR
      t.ping = 'rust-lang/infra' OR
      t.ping = 'rust-lang/dev-tools'
    );

-- remove new teams
DELETE FROM teams t
WHERE
  t.ping = 'rust-lang/cargo' OR
  t.ping = 'rust-lang/infra' OR
  t.ping = 'rust-lang/dev-tools';

-- add peschkaj back to docs team
INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'rust-lang/docs' AND
    u.login = 'peschkaj';
