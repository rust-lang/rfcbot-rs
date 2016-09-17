use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, FilterDsl, LoadDsl, Queryable, SaveChangesDsl, Table};

use super::schema::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[insertable_into(fcp_proposal)]
pub struct NewFcpProposal<'a> {
    pub fk_issue: i32,
    pub fk_initiator: i32,
    pub fk_initiating_comment: i32,
    pub disposition: &'a str,
    pub fk_bot_tracking_comment: i32,
    pub fcp_start: Option<NaiveDateTime>,
    pub fcp_closed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable, Serialize)]
#[changeset_for(fcp_proposal, treat_none_as_null="true")]
pub struct FcpProposal {
    pub id: i32,
    pub fk_issue: i32,
    pub fk_initiator: i32,
    pub fk_initiating_comment: i32,
    pub disposition: String,
    pub fk_bot_tracking_comment: i32,
    pub fcp_start: Option<NaiveDateTime>,
    pub fcp_closed: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[insertable_into(fcp_review_request)]
pub struct NewFcpReviewRequest {
    pub fk_proposal: i32,
    pub fk_reviewer: i32,
    pub reviewed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable, Serialize)]
#[changeset_for(fcp_review_request, treat_none_as_null="true")]
pub struct FcpReviewRequest {
    pub id: i32,
    pub fk_proposal: i32,
    pub fk_reviewer: i32,
    pub reviewed: bool,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[insertable_into(fcp_concern)]
pub struct NewFcpConcern<'a> {
    pub fk_proposal: i32,
    pub fk_initiator: i32,
    pub fk_resolved_comment: Option<i32>,
    pub name: &'a str,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[changeset_for(fcp_concern, treat_none_as_null="true")]
pub struct FcpConcern {
    pub id: i32,
    pub fk_proposal: i32,
    pub fk_initiator: i32,
    pub fk_resolved_comment: Option<i32>,
    pub name: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[insertable_into(rfc_feedback_request)]
pub struct NewFeedbackRequest {
    pub fk_initiator: i32,
    pub fk_requested: i32,
    pub fk_issue: i32,
    pub fk_feedback_comment: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[changeset_for(rfc_feedback_request, treat_none_as_null="true")]
pub struct FeedbackRequest {
    pub id: i32,
    pub fk_initiator: i32,
    pub fk_requested: i32,
    pub fk_issue: i32,
    pub fk_feedback_comment: Option<i32>,
}
