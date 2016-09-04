INSERT INTO teams (name, ping, label) VALUES ('Documentation', 'rust-lang/docs', 'T-doc');

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'rust-lang/compiler' AND
    u.login = 'jseyfriend';

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'rust-lang/docs' AND (
    u.login = 'steveklabnik' OR
    u.login = 'GuillaumeGomez' OR
    u.login = 'jonathandturner' OR
    u.login = 'peschkaj'
);

DELETE FROM memberships m USING githubuser u, teams t
WHERE
    m.fk_member = u.id AND
    m.fk_team = t.id AND (
        (u.login = 'Gankro' AND t.ping = 'rust-lang/libs') OR
        (u.login = 'huonw' AND t.ping = 'rust-lang/core') OR
        (u.login = 'huonw' AND t.ping = 'rust-lang/lang') OR
        (u.login = 'huonw' AND t.ping = 'rust-lang/libs')
    );
