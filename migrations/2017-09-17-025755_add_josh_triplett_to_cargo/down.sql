DELETE FROM memberships m USING githubuser u, teams t
WHERE
    m.fk_member = u.id AND
    m.fk_team = t.id AND
    u.login = 'JoshTriplett' AND t.ping = 'rust-lang/cargo';
