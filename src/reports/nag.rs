use reports::teams::MEMBERSHIP;

pub fn all_team_members() -> Vec<&'static str> {
    MEMBERSHIP.keys().map(|m| *m).collect()
}
