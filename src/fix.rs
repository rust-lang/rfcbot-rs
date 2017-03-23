use diesel;
use diesel::prelude::*;
use diesel::types::{Integer, VarChar};

use github::{update_issue, update_pr};

use DB_POOL;

pub fn fix() {
    for (repo, number) in get_duplicate_issues() {
        delete_duplicate_issues(&repo, number);
        update_issue(&repo, number).expect("Failed to update issue");
    }

    for (repo, number) in get_duplicate_prs() {
        delete_duplicate_prs(&repo, number);
        update_pr(&repo, number).expect("Failed to update issue");
    }
}

fn get_duplicate_issues() -> Vec<(String, i32)> {
    use diesel::expression::dsl::*;

    let conn = DB_POOL.get().expect("Failed to acquire connection");
    diesel::select(sql::<(VarChar, Integer)>("repository, number \
        FROM issue \
        GROUP BY number, repository HAVING COUNT(*) > 1"))
        .load::<(String, i32)>(&*conn)
        .expect("Failed to load duplicates")
}

fn delete_duplicate_issues(repo: &str, issue: i32) {
    use domain::schema::issue;
    use domain::schema::issuecomment;

    let conn = DB_POOL.get().expect("Failed to acquire connection");

    // Delete comments first to avoid foreign key problems
    let ids = issue::table.select(issue::id)
        .filter(issue::number.eq(issue))
        .filter(issue::repository.eq(repo));
    diesel::delete(issuecomment::table
                   .filter(issuecomment::fk_issue.eq_any(ids)))
        .execute(&*conn)
        .expect("Failed to delete issue comments");

    // Then delete actual issues
    diesel::delete(issue::table
                   .filter(issue::number.eq(issue))
                   .filter(issue::repository.eq(repo)))
        .execute(&*conn)
        .expect("Failed to delete issue");
}

fn get_duplicate_prs() -> Vec<(String, i32)> {
    use diesel::expression::dsl::*;

    let conn = DB_POOL.get().expect("Failed to acquire connection");
    diesel::select(sql::<(VarChar, Integer)>("repository, number \
        FROM pullrequest \
        GROUP BY number, repository HAVING COUNT(*) > 1"))
        .load::<(String, i32)>(&*conn)
        .expect("Failed to load duplicates")
}

fn delete_duplicate_prs(repo: &str, issue: i32) {
    use domain::schema::pullrequest;

    let conn = DB_POOL.get().expect("Failed to acquire connection");

    // Then delete actual issues
    diesel::delete(pullrequest::table
                   .filter(pullrequest::number.eq(issue))
                   .filter(pullrequest::repository.eq(repo)))
        .execute(&*conn)
        .expect("Failed to delete pr");
}
