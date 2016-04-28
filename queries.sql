# PRs opened per day
SELECT pr.created_at::date as d, COUNT(*)
FROM pullrequest pr
GROUP BY d
ORDER BY d DESC

# PRs closed per day
SELECT pr.closed_at::date as d, COUNT(*)
FROM pullrequest pr
WHERE pr.closed_at IS NOT NULL
GROUP BY d
ORDER BY d DESC

# PRs merged per day
SELECT pr.merged_at::date as d, COUNT(*)
FROM pullrequest pr
WHERE pr.merged_at IS NOT NULL
GROUP BY d
ORDER BY d DESC

# Time PRs are open before closing, grouped by the week in which they were closed
SELECT
  COUNT(*),
  AVG(EXTRACT(EPOCH FROM closed_at) -
    EXTRACT(EPOCH FROM created_at))
    / (60 * 60 * 24),
  EXTRACT(ISOYEAR FROM closed_at)::text || '-' ||
    EXTRACT(WEEK FROM closed_at)::text || '-6' AS iso_closed_week
FROM pullrequest
WHERE
  closed_at IS NOT NULL AND
  closed_at >= NOW() - INTERVAL '90 days' AND
  closed_at <= NOW()
GROUP BY iso_closed_week
ORDER BY iso_closed_week DESC

# age of still-open PRs
SELECT
  AVG(EXTRACT(EPOCH FROM (now() - created_at))) / (60 * 60 * 24)
FROM pullrequest
WHERE closed_at IS NULL

# number of '@bors: retry' per PR
SELECT ic.fk_issue, COUNT(ic.*)
FROM issuecomment ic
WHERE ic.body LIKE '%@bors%retry%'
GROUP BY ic.fk_issue
ORDER BY COUNT(ic.*) DESC

# issues opened per day
SELECT i.created_at::date as d, COUNT(*)
FROM issue i
GROUP BY d
ORDER BY d DESC

# issues closed per day
SELECT i.closed_at::date as d, COUNT(*)
FROM issue i
WHERE i.closed_at IS NOT NULL
GROUP BY d
ORDER BY d DESC

# time issues are open before closing (NEEDS WORK AND CLARIFICATION)
SELECT
  i.number,
  i.closed_at,
  (EXTRACT(EPOCH FROM (i.closed_at - i.created_at))) as minutes_open
FROM issue i
WHERE
  i.closed_at IS NOT NULL AND
  i.created_at IS NOT NULL
ORDER BY i.closed_at desc

# age of still-open issues
SELECT
  i.number,
  i.created_at,
  (EXTRACT(EPOCH FROM (now() - i.created_at)) / 60) as minutes_open
FROM issue i
WHERE i.closed_at IS NULL
ORDER BY i.created_at ASC

# number of open P-high issues
SELECT
  i.number,
  i.labels
FROM issue i
WHERE
  NOT i.open AND
  'P-high' = ANY (i.labels)

# number of regression issues
SELECT
  i.number,
  i.labels
FROM issue i
WHERE
  NOT i.open AND
  'regression-from-stable-to-beta' = ANY (i.labels) OR
  'regression-from-stable-to-nightly' = ANY (i.labels) OR
  'regression-from-stable-to-stable' = ANY (i.labels)

# 'auto-*' buildbot completion times
SELECT b.builder_name, AVG(b.duration_secs) / 60 as duration_minutes
FROM build b
WHERE
  b.successful AND
  b.builder_name LIKE 'auto-%'
GROUP BY b.builder_name
ORDER BY AVG(b.duration_secs) DESC

###############################################
# everything below this line is unimplemented
###############################################

# build-bot specific failures in last 2 days (from bors retries)
SELECT
  regexp_replace(
    replace(ic.body, ':broken_heart: Test failed - [', ''),
    '\]\(.+', '') as failed_bot,
  COUNT(*)
FROM issuecomment ic, githubuser u
WHERE
  ic.created_at > CURRENT_DATE - INTERVAL '2 days' AND
  u.login = 'bors' AND
  u.id = ic.fk_user AND
  ic.body LIKE '%Test failed - %' AND
  ic.fk_issue IN (SELECT pr.number
	FROM issuecomment ic, pullrequest pr
	WHERE
	  ic.body LIKE '%@bors%retry%' AND
	  pr.number = ic.fk_issue)
GROUP BY failed_bot
ORDER BY COUNT(*) DESC

# windows builtbot failures in last 7 days
SELECT
  regexp_replace(
    replace(ic.body, ':broken_heart: Test failed - [', ''),
    '\]\(.+', '') as failed_bot,
  'https://github.com/rust-lang/rust/pull/' || ic.fk_issue as pr_url,
  regexp_replace(
    replace(ic.body, ':broken_heart: Test failed - [', ''),
    '.+\]\(', '') as log_url
FROM issuecomment ic, githubuser u
WHERE
  ic.created_at > CURRENT_DATE - INTERVAL '1 week' AND
  u.login = 'bors' AND
  u.id = ic.fk_user AND
  ic.body LIKE '%Test failed - %' AND
  ic.fk_issue IN (SELECT pr.number
	FROM issuecomment ic, pullrequest pr
	WHERE
	  ic.body LIKE '%@bors%retry%' AND
	  pr.number = ic.fk_issue)
ORDER BY failed_bot
