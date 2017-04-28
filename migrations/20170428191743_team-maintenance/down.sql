INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE (
    t.ping = 'rust-lang/core' AND
    (u.login = 'pcwalton')
  )
  OR (
    t.ping = 'rust-lang/compiler' AND
    (u.login = 'bkoropoff' OR u.login = 'dotdash')
  );


DELETE FROM memberships m USING githubuser u, teams t
WHERE
    m.fk_member = u.id AND
    m.fk_team = t.id AND
    (
    (u.login = 'carols10cents' AND t.ping = 'rust-lang/core') OR
    (u.login = 'nrc' AND t.ping = 'rust-lang/core') OR
    (u.login = 'jseyfried' AND t.ping = 'rust-lang/compiler')
    );
