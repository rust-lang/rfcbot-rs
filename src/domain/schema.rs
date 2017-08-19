table! {
    build (id) {
        id -> Int4,
        build_id -> Varchar,
        env -> Text,
        successful -> Bool,
        message -> Text,
        duration_secs -> Nullable<Int4>,
        start_time -> Nullable<Timestamp>,
        end_time -> Nullable<Timestamp>,
        builder_name -> Text,
        job_id -> Text,
        os -> Text,
    }
}

table! {
    fcp_concern (id) {
        id -> Int4,
        fk_proposal -> Int4,
        fk_initiator -> Int4,
        fk_resolved_comment -> Nullable<Int4>,
        name -> Varchar,
        fk_initiating_comment -> Int4,
    }
}

table! {
    fcp_proposal (id) {
        id -> Int4,
        fk_issue -> Int4,
        fk_initiator -> Int4,
        fk_initiating_comment -> Int4,
        disposition -> Varchar,
        fk_bot_tracking_comment -> Int4,
        fcp_start -> Nullable<Timestamp>,
        fcp_closed -> Bool,
    }
}

table! {
    fcp_review_request (id) {
        id -> Int4,
        fk_proposal -> Int4,
        fk_reviewer -> Int4,
        reviewed -> Bool,
    }
}

table! {
    githubsync (id) {
        id -> Int4,
        successful -> Bool,
        ran_at -> Timestamp,
        message -> Nullable<Varchar>,
    }
}

table! {
    githubuser (id) {
        id -> Int4,
        login -> Varchar,
    }
}

table! {
    issue (id) {
        id -> Int4,
        number -> Int4,
        fk_milestone -> Nullable<Int4>,
        fk_user -> Int4,
        fk_assignee -> Nullable<Int4>,
        open -> Bool,
        is_pull_request -> Bool,
        title -> Varchar,
        body -> Varchar,
        locked -> Bool,
        closed_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        labels -> Array<Text>,
        repository -> Varchar,
    }
}

table! {
    issuecomment (id) {
        id -> Int4,
        fk_issue -> Int4,
        fk_user -> Int4,
        body -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        repository -> Varchar,
    }
}

table! {
    memberships (id) {
        id -> Int4,
        fk_member -> Int4,
        fk_team -> Int4,
    }
}

table! {
    milestone (id) {
        id -> Int4,
        number -> Int4,
        open -> Bool,
        title -> Varchar,
        description -> Nullable<Varchar>,
        fk_creator -> Int4,
        open_issues -> Int4,
        closed_issues -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        closed_at -> Nullable<Timestamp>,
        due_on -> Nullable<Timestamp>,
        repository -> Varchar,
    }
}

table! {
    pullrequest (id) {
        id -> Int4,
        number -> Int4,
        state -> Varchar,
        title -> Varchar,
        body -> Nullable<Varchar>,
        fk_assignee -> Nullable<Int4>,
        fk_milestone -> Nullable<Int4>,
        locked -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        closed_at -> Nullable<Timestamp>,
        merged_at -> Nullable<Timestamp>,
        commits -> Int4,
        additions -> Int4,
        deletions -> Int4,
        changed_files -> Int4,
        repository -> Varchar,
    }
}

table! {
    release (id) {
        id -> Int4,
        date -> Date,
        released -> Bool,
    }
}

table! {
    rfc_feedback_request (id) {
        id -> Int4,
        fk_initiator -> Int4,
        fk_requested -> Int4,
        fk_issue -> Int4,
        fk_feedback_comment -> Nullable<Int4>,
    }
}

table! {
    teams (id) {
        id -> Int4,
        name -> Varchar,
        ping -> Varchar,
        label -> Varchar,
    }
}
