-- TODO: is there a reasonable rollback procedure? I think we could undo the
-- data type changes, but it would likely mean deleting some comments from the
-- database that can no longer be represented. Is that a reasonable tradeoff?
begin; commit;
