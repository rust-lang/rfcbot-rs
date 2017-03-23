CREATE UNIQUE INDEX issue_repo_number ON issue(repository, number);
CREATE UNIQUE INDEX pullrequest_repo_number ON pullrequest(repository, number);
