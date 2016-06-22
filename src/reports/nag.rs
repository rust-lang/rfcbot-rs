use reports::teams::MEMBERSHIP;

pub fn all_team_members() -> Vec<&'static str> {
    let mut members = MEMBERSHIP.keys().map(|m| *m).collect::<Vec<_>>();

    members.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    members
}
