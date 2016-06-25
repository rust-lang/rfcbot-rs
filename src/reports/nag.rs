use diesel::expression::dsl::*;
use diesel::prelude::*;
use diesel::select;
use diesel::types::VarChar;

use DB_POOL;
use error::DashResult;

pub fn all_team_members() -> DashResult<Vec<String>> {
    let conn = try!(DB_POOL.get());

    // waiting on associations to get this into proper typed queries

    Ok(try!(select(sql::<VarChar>("\
        DISTINCT u.login \
        FROM githubuser u, memberships m \
        WHERE u.id = m.fk_member \
        ORDER BY u.login"))
        .load(&*conn)))
}

pub fn individual_nags(_: &str) -> DashResult<Vec<String>> {
    Ok(Vec::new())
}
