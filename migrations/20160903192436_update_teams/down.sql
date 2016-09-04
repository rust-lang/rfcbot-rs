DELETE FROM memberships m USING githubuser u, teams t
WHERE
    m.fk_member = u.id AND
    m.fk_team = t.id AND
    t.ping = 'rust-lang/compiler' AND
    u.login = 'jseyfriend';

DELETE FROM memberships m USING githubuser u, teams t
WHERE
    m.fk_member = u.id AND
    m.fk_team = t.id AND
    t.ping = 'rust-lang/docs' AND (
    u.login = 'steveklabnik' OR
    u.login = 'GuillaumeGomez' OR
    u.login = 'jonathandturner' OR
    u.login = 'peschkaj'
);

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE
    (u.login = 'Gankro' AND t.ping = 'rust-lang/libs') OR
    (u.login = 'huonw' AND t.ping = 'rust-lang/core') OR
    (u.login = 'huonw' AND t.ping = 'rust-lang/lang') OR
    (u.login = 'huonw' AND t.ping = 'rust-lang/libs');

DELETE FROM teams t
WHERE
    t.ping = 'rust-lang/docs';
