use diesel::ExpressionMethods;

use super::schema::*;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[insertable_into(build)]
#[changeset_for(build)]
pub struct Build {
    pub number: i32,
    pub builder_name: String,
    pub successful: bool,
    pub message: String,
    pub duration_secs: Option<i32>,
}

// TODO record end time
