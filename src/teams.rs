// TODO maybe pull from https://github.com/rust-lang/rust-www/blob/master/_data/team.yml instead

use std::collections::BTreeMap;

use diesel::prelude::*;
use toml;

use super::DB_POOL;
use domain::github::GitHubUser;
use error::*;

lazy_static! {
    pub static ref TEAMS: Teams = {
        let teams_file =
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/teams.toml"));
        let teams_from_file: TeamsFromFile =
            toml::from_str(teams_file).expect("couldn't parse teams");

        let mut teams = BTreeMap::new();

        for (label, team_from_file) in teams_from_file {
            let label = TeamLabel(label);
            let team = team_from_file.validate()
                .expect("unable to verify team member from database.
if you're running this for tests, make sure you've pulled github users from prod");
            teams.insert(label, team);
        }

        teams
    };
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TeamLabel(pub String);

type TeamsFromFile = BTreeMap<String, TeamFromFile>;
pub type Teams = BTreeMap<TeamLabel, Team>;

#[derive(Debug, Deserialize)]
struct TeamFromFile {
  name: String,
  ping: String,
  members: Vec<String>,
}

impl TeamFromFile {
  pub fn validate(self) -> DashResult<Team> {
    use domain::schema::githubuser::dsl::*;
    let conn = &*(DB_POOL.get()?);

    // bail if they don't exist, but we don't want to actually keep the id in ram
    for member_login in &self.members {
      match githubuser
        .filter(login.eq(member_login))
        .first::<GitHubUser>(conn) {
            Ok(_) => (),
            Err(why) => {
                error!("unable to find {} in database: {:?}", member_login, why);
                return Err(why.into());
            }
        }
    }

    Ok(Team {
      name: self.name,
      ping: self.ping,
      member_logins: self.members,
    })
  }
}

pub struct Team {
  pub name: String,
  pub ping: String,
  pub member_logins: Vec<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn team_members_exist() {
        for (label, team) in TEAMS.iter() {
            println!("found team {:?}", label);
        }
    }
}
