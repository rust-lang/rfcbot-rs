begin;

-- Update all issuecomment referencing columns to bigints

alter table issuecomment alter column id set data type bigint;

alter table fcp_concern alter column fk_initiating_comment set data type bigint;
alter table fcp_concern alter column fk_resolved_comment set data type bigint;

alter table fcp_proposal alter column fk_bot_tracking_comment set data type bigint;
alter table fcp_proposal alter column fk_initiating_comment set data type bigint;

alter table poll alter column fk_bot_tracking_comment set data type bigint;
alter table poll alter column fk_initiating_comment set data type bigint;

alter table rfc_feedback_request alter column fk_feedback_comment set data type bigint;

-- Drop all foreign key constraints so that we can update values one by one.

alter table fcp_concern drop constraint fcp_concern_fk_initiating_comment_fkey;
alter table fcp_concern drop constraint fcp_concern_fk_resolved_comment_fkey;

alter table fcp_proposal drop constraint fcp_proposal_fk_bot_tracking_comment_fkey;
alter table fcp_proposal drop constraint fcp_proposal_fk_initiating_comment_fkey;

alter table poll drop constraint poll_fk_bot_tracking_comment_fkey;
alter table poll drop constraint poll_fk_initiating_comment_fkey;

alter table rfc_feedback_request drop constraint rfc_feedback_request_fk_feedback_comment_fkey;

-- Then update all the values to be naturally stored, not u32s mapped into i32.

update fcp_concern set fk_initiating_comment = fk_initiating_comment::bigint + 4294967296 where fk_initiating_comment < 0;
update fcp_concern set fk_resolved_comment = fk_resolved_comment::bigint + 4294967296 where fk_resolved_comment < 0;

update fcp_proposal set fk_bot_tracking_comment = fk_bot_tracking_comment::bigint + 4294967296 where fk_bot_tracking_comment < 0;
update fcp_proposal set fk_initiating_comment = fk_initiating_comment::bigint + 4294967296 where fk_initiating_comment < 0;

update poll set fk_bot_tracking_comment = fk_bot_tracking_comment::bigint + 4294967296 where fk_bot_tracking_comment < 0;
update poll set fk_initiating_comment = fk_initiating_comment::bigint + 4294967296 where fk_initiating_comment < 0;

update rfc_feedback_request set fk_feedback_comment = fk_feedback_comment::bigint + 4294967296 where fk_feedback_comment < 0;

update issuecomment set id = id::bigint + 4294967296 where id < 0;

-- Then re-create the foreign key constraints.

alter table fcp_concern add CONSTRAINT fcp_concern_fk_initiating_comment_fkey FOREIGN KEY (fk_initiating_comment) REFERENCES issuecomment(id);
alter table fcp_concern add CONSTRAINT fcp_concern_fk_resolved_comment_fkey FOREIGN KEY (fk_resolved_comment) REFERENCES issuecomment(id);
alter table fcp_proposal add CONSTRAINT fcp_proposal_fk_bot_tracking_comment_fkey FOREIGN KEY (fk_bot_tracking_comment) REFERENCES issuecomment(id);
alter table fcp_proposal add CONSTRAINT fcp_proposal_fk_initiating_comment_fkey FOREIGN KEY (fk_initiating_comment) REFERENCES issuecomment(id);
alter table poll add CONSTRAINT poll_fk_bot_tracking_comment_fkey FOREIGN KEY (fk_bot_tracking_comment) REFERENCES issuecomment(id);
alter table poll add CONSTRAINT poll_fk_initiating_comment_fkey FOREIGN KEY (fk_initiating_comment) REFERENCES issuecomment(id);
alter table rfc_feedback_request add CONSTRAINT rfc_feedback_request_fk_feedback_comment_fkey FOREIGN KEY (fk_feedback_comment) REFERENCES issuecomment(id);

commit;
