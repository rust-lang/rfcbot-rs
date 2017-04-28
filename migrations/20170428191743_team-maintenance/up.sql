-- add carols10cents and nrc to core team
-- add jseyfried to compiler team
INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE (
    t.ping = 'rust-lang/core' AND
    (u.login = 'carols10cents' OR u.login = 'nrc')
  )
  OR (
    t.ping = 'rust-lang/compiler' AND
    u.login = 'jseyfried'
  );

-- remove pcwalton from core team
-- remove bkoropoff and dotdash from compiler team
DELETE FROM memberships m USING githubuser u, teams t
WHERE
    m.fk_member = u.id AND
    m.fk_team = t.id AND
    (
    (u.login = 'pcwalton' AND t.ping = 'rust-lang/core') OR
    (u.login = 'bkoropoff' AND t.ping = 'rust-lang/compiler') OR
    (u.login = 'dotdash' AND t.ping = 'rust-lang/compiler')
    );
