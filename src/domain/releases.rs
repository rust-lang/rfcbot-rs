use chrono::NaiveDate;
use diesel::ExpressionMethods;

use super::schema::*;

#[derive(AsChangeset, Clone, Debug, Deserialize, Eq, Insertable,
         Ord, PartialEq, PartialOrd, Queryable, Serialize)]
#[table_name="release"]
pub struct Release {
    pub date: NaiveDate,
    pub released: bool,
}
