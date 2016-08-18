use diesel::{ExpressionMethods, FilterDsl, LoadDsl, Queryable, SaveChangesDsl, Table};

use super::schema::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[insertable_into(fcp_proposal)]
pub struct NewFcpProposal<'a> {
    fk_issue: i32,
    fk_initiator: i32,
    fk_initiating_comment: i32,
    disposition: &'a str,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[changeset_for(fcp_proposal, treat_none_as_null="true")]
pub struct FcpProposal {
    id: i32,
    fk_issue: i32,
    fk_initiator: i32,
    fk_initiating_comment: i32,
    disposition: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[insertable_into(fcp_review_request)]
pub struct NewFcpReviewRequest {
    fk_proposal: i32,
    fk_reviewer: i32,
    fk_reviewed_comment: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[changeset_for(fcp_review_request, treat_none_as_null="true")]
pub struct FcpReviewRequest {
    id: i32,
    fk_proposal: i32,
    fk_reviewer: i32,
    fk_reviewed_comment: Option<i32>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[insertable_into(fcp_concern)]
pub struct NewFcpConcern<'a> {
    fk_proposal: i32,
    fk_initiator: i32,
    fk_resolved_comment: Option<i32>,
    name: &'a str,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[changeset_for(fcp_concern, treat_none_as_null="true")]
pub struct FcpConcern {
    id: i32,
    fk_proposal: i32,
    fk_initiator: i32,
    fk_resolved_comment: Option<i32>,
    name: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[insertable_into(rfc_feedback_request)]
pub struct NewFeedbackRequest {
    fk_iniatiator: i32,
    fk_requested: i32,
    fk_feedback_comment: Option<i32>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Queryable)]
#[changeset_for(rfc_feedback_request, treat_none_as_null="true")]
pub struct FeedbackRequest {
    id: i32,
    fk_initiator: i32,
    fk_requested: i32,
    fk_feedback_comment: Option<i32>,
}
