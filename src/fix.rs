use diesel;
use diesel::prelude::*;
use diesel::types::{Integer, VarChar};

use github::update_issue;

use DB_POOL;

pub fn fix() {
    for (repo, number) in get_duplicate_issues() {
        delete_duplicates(&repo, number);
        update_issue(&repo, number).expect("Failed to update issue");
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

fn delete_duplicates(repo: &str, issue: i32) {
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
