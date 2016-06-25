SELECT i.repository, i.number, i.title
FROM issue i
WHERE
i.id IN (
	-- get all nag issues written by others on the same team(s)
	SELECT i.id
	FROM issue i, issuecomment ic
	WHERE
	  (ic.body LIKE 'f?%' || '@nrc%' OR
	  ic.body LIKE 'f?%' || '@rust-lang/lang%') AND
	  ic.fk_issue = i.id AND
	  i.fk_user IN (
		-- get all potential issue authors on the same team
		SELECT DISTINCT u.id
		FROM githubuser u, githubuser u2, memberships m, memberships m2
		WHERE
		  u2.login = 'nrc' AND /* this needs to be a bind param */
		  u2.id = m2.fk_member AND
		  m2.fk_team = m.fk_team AND
		  u.id = m.fk_member))
