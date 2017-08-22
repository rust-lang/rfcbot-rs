DELETE FROM memberships m USING githubuser u
WHERE
    m.fk_member = u.id AND
    u.login = 'brson';
