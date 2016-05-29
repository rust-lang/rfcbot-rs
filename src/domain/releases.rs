use chrono::NaiveDate;
use diesel::ExpressionMethods;

use super::schema::*;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable, Serialize)]
#[insertable_into(release)]
#[changeset_for(release)]
pub struct Release {
    pub date: NaiveDate,
    pub released: bool,
}
