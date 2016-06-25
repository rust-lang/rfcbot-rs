CREATE TABLE teams (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    ping VARCHAR NOT NULL,
    label VARCHAR NOT NULL
);

CREATE TABLE memberships (
    id SERIAL PRIMARY KEY,
    fk_member INTEGER REFERENCES githubuser (id),
    fk_team INTEGER REFERENCES teams (id)
);

INSERT INTO teams (name, ping, label) VALUES
    ('Core', 'rust-lang/core', 'T-core'),
    ('Language', 'rust-lang/lang', 'T-lang'),
    ('Libraries', 'rust-lang/libs', 'T-libs'),
    ('Compiler', 'rust-lang/compiler', 'T-compiler'),
    ('Tools', 'rust-lang/tools', 'T-tools');

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'rust-lang/core' AND (
    u.login = 'brson' OR
    u.login = 'alexcrichton' OR
    u.login = 'wycats' OR
    u.login = 'steveklabnik' OR
    u.login = 'nikomatsakis' OR
    u.login = 'aturon' OR
    u.login = 'pcwalton' OR
    u.login = 'huonw' OR
    u.login = 'erickt'
);

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'rust-lang/lang' AND (
    u.login = 'eddyb' OR
    u.login = 'nrc' OR
    u.login = 'pnkfelix' OR
    u.login = 'nikomatsakis' OR
    u.login = 'aturon' OR
    u.login = 'huonw'
);

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'rust-lang/libs' AND (
    u.login = 'brson' OR
    u.login = 'Gankro' OR
    u.login = 'alexcrichton' OR
    u.login = 'sfackler' OR
    u.login = 'BurntSushi' OR
    u.login = 'Kimundi' OR
    u.login = 'aturon' OR
    u.login = 'huonw'
);

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'rust-lang/compiler' AND (
    u.login = 'arielb1' OR
    u.login = 'eddyb' OR
    u.login = 'nrc' OR
    u.login = 'pnkfelix' OR
    u.login = 'bkoropoff' OR
    u.login = 'nikomatsakis' OR
    u.login = 'dotdash' OR
    u.login = 'michaelwoerister' OR
    u.login = 'Aatch'
);

INSERT INTO memberships (fk_member, fk_team)
SELECT u.id, t.id
FROM githubuser u, teams t
WHERE t.ping = 'rust-lang/tools' AND (
    u.login = 'brson' OR
    u.login = 'nrc' OR
    u.login = 'alexcrichton' OR
    u.login = 'vadimcn' OR
    u.login = 'wycats' OR
    u.login = 'michaelwoerister'
);
