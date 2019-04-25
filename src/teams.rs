use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use diesel::prelude::*;

use super::DB_POOL;
use crate::domain::github::GitHubUser;
use crate::error::*;
use crate::github::GH;

const UPDATE_CONFIG_EVERY_MIN: u64 = 5;

//==============================================================================
// Public API
//==============================================================================

type TeamsMap = BTreeMap<TeamLabel, Team>;

lazy_static! {
    pub static ref SETUP: Arc<RwLock<RfcbotConfig>> =
        Arc::new(RwLock::new(read_rfcbot_cfg_validated()));
}

#[derive(Debug, Deserialize)]
pub struct RfcbotConfig {
    #[serde(default)]
    include_rust_team: bool,
    fcp_behaviors: BTreeMap<String, FcpBehavior>,
    teams: RfcbotTeams,
    #[serde(skip)]
    cached_teams: TeamsMap,
}

impl RfcbotConfig {
    /// Retrive an iterator over all the team labels.
    pub fn team_labels(&self) -> impl Iterator<Item = &TeamLabel> { self.teams().map(|(k, _)| k) }

    /// Retrive an iterator over all the (team label, team) pairs.
    pub fn teams(&self) -> impl Iterator<Item = (&TeamLabel, &Team)> {
        match &self.teams {
            RfcbotTeams::Local(teams) => teams.iter(),
            RfcbotTeams::Remote { .. } => self.cached_teams.iter(),
        }
    }

    /// Are we allowed to auto-close issues after F-FCP in this repo?
    pub fn should_ffcp_auto_close(&self, repo: &str) -> bool {
        self.fcp_behaviors
            .get(repo)
            .map(|fcp| fcp.close)
            .unwrap_or_default()
    }

    /// Are we allowed to auto-postpone issues after F-FCP in this repo?
    pub fn should_ffcp_auto_postpone(&self, repo: &str) -> bool {
        self.fcp_behaviors
            .get(repo)
            .map(|fcp| fcp.postpone)
            .unwrap_or_default()
    }

    // Update the list of teams from external sources, if needed
    fn update(&mut self) -> Result<(), DashError> {
        #[derive(Deserialize)]
        struct ToDeserialize {
            teams: TeamsMap,
        }
        if let RfcbotTeams::Remote { ref url } = &self.teams {
            let de: ToDeserialize = ::reqwest::get(url)?.error_for_status()?.json()?;
            self.cached_teams = de.teams;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct FcpBehavior {
    #[serde(default)]
    close: bool,
    #[serde(default)]
    postpone: bool,
}

// This enum definition mixes both struct-style and tuple-style variants: this is intentionally
// done to get the wanted deserialization behavior from serde. Since this is an untagged enum from
// serde's point of view it will deserialize a RfcbotTeams::Remote when it encounters a key named
// url with a string in it, otherwise the normal team map.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RfcbotTeams {
    Local(TeamsMap),
    Remote { url: String },
}

#[derive(Debug, Deserialize)]
pub struct Team {
    name: String,
    ping: String,
    members: Vec<String>,
}

impl Team {
    pub fn ping(&self) -> &str { &self.ping }

    pub fn member_logins(&self) -> impl Iterator<Item = &str> {
        self.members.iter().map(std::string::String::as_str)
    }
}

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize)]
#[serde(transparent)]
pub struct TeamLabel(pub String);

pub fn start_updater_thread() {
    let _ = crate::utils::spawn_thread("teams updater", UPDATE_CONFIG_EVERY_MIN, || {
        let mut teams = SETUP.write().unwrap();
        teams.update()?;
        for (_name, team) in teams.teams() {
            team.validate()?;
        }
        Ok(())
    });
}

//==============================================================================
// Implementation details
//==============================================================================

/// Read the validated `rfcbot.toml` configuration file.
fn read_rfcbot_cfg_validated() -> RfcbotConfig {
    let cfg = read_rfcbot_cfg();

    cfg.teams().map(|(_, v)| v).for_each(|team| {
        team.validate().expect(
            "unable to verify team member from database.
if you're running this for tests, make sure you've pulled github users from prod",
        )
    });

    cfg
}

/// Read the unprocessed `rfcbot.toml` configuration file.
fn read_rfcbot_cfg() -> RfcbotConfig {
    let mut config = read_rfcbot_cfg_from(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/rfcbot.toml"
    )));
    config.update().expect("couldn't update the configuration!");
    config
}

fn read_rfcbot_cfg_from(input: &str) -> RfcbotConfig {
    toml::from_str(input).expect("couldn't parse rfcbot.toml!")
}

impl Team {
    fn validate(&self) -> DashResult<()> {
        use crate::domain::schema::githubuser::dsl::*;
        let conn = &*(DB_POOL.get()?);
        let gh = &*(GH);

        // bail if they don't exist, but we don't want to actually keep the id in ram
        for member_login in self.member_logins() {
            if githubuser
                .filter(login.eq(member_login))
                .first::<GitHubUser>(conn)
                .is_err()
            {
                crate::github::handle_user(&conn, &gh.get_user(member_login)?)?;
                info!("loaded into the database user {}", member_login);
            }
        }

        Ok(())
    }
}

//==============================================================================
// Tests
//==============================================================================

#[cfg(test)]
pub mod test {
    use super::*;

    lazy_static! {
        pub static ref TEST_SETUP: RfcbotConfig = read_rfcbot_cfg_from(
            r#"
[fcp_behaviors]

[fcp_behaviors."rust-lang/alpha"]
close = true
postpone = true

[fcp_behaviors."foobar/beta"]
close = false

[fcp_behaviors."bazquux/gamma"]
postpone = false

[fcp_behaviors."wibble/epsilon"]

[teams]

[teams.T-avengers]
name = "The Avengers"
ping = "marvel/avengers"
members = [
  "hulk",
  "thor",
  "thevision",
  "blackwidow",
  "spiderman",
  "captainamerica",
]

[teams.justice-league]
name = "Justice League of America"
ping = "dc-comics/justice-league"
members = [
  "superman",
  "wonderwoman",
  "aquaman",
  "batman",
  "theflash"
]
"#
        );
    }

    #[test]
    fn setup_parser_correct() {
        let cfg = &*TEST_SETUP;

        // Labels are correct:
        assert_eq!(
            cfg.team_labels().map(|tl| tl.0.clone()).collect::<Vec<_>>(),
            vec!["T-avengers", "justice-league"]
        );

        // Teams are correct:
        let map: BTreeMap<_, _> = cfg.teams().map(|(k, v)| (k.0.clone(), v)).collect();

        let avengers = map.get("T-avengers").unwrap();
        //assert_eq!(avengers.name, "The Avengers");
        //assert_eq!(avengers.ping, "marvel/avengers");
        assert_eq!(
            avengers.member_logins().collect::<Vec<_>>(),
            vec![
                "hulk",
                "thor",
                "thevision",
                "blackwidow",
                "spiderman",
                "captainamerica"
            ]
        );

        let jsa = map.get("justice-league").unwrap();
        //assert_eq!(jsa.name, "Justice League of America");
        //assert_eq!(jsa.ping, "dc-comics/justice-league");
        assert_eq!(
            jsa.member_logins().collect::<Vec<_>>(),
            vec!["superman", "wonderwoman", "aquaman", "batman", "theflash"]
        );

        // Random non-existent team does not exist:
        assert!(map.get("random").is_none());

        // FFCP behavior correct:
        assert!(cfg.should_ffcp_auto_close("rust-lang/alpha"));
        assert!(cfg.should_ffcp_auto_postpone("rust-lang/alpha"));
        assert!(!cfg.should_ffcp_auto_close("foobar/beta"));
        assert!(!cfg.should_ffcp_auto_postpone("foobar/beta"));
        assert!(!cfg.should_ffcp_auto_close("bazquux/gamma"));
        assert!(!cfg.should_ffcp_auto_postpone("bazquux/gamma"));
        assert!(!cfg.should_ffcp_auto_close("wibble/epsilon"));
        assert!(!cfg.should_ffcp_auto_postpone("wibble/epsilon"));
        assert!(!cfg.should_ffcp_auto_close("random"));
        assert!(!cfg.should_ffcp_auto_postpone("random"));
    }

    #[test]
    fn cfg_file_wellformed() {
        // Just parse it and ensure that we get no panics for now!
        // This is a crap test; but, better than nothing.
        let _ = read_rfcbot_cfg();
    }

    #[test]
    fn team_members_exist() {
        crate::utils::setup_test_env();
        let setup = SETUP.read().unwrap();
        for (label, _) in setup.teams() {
            println!("found team {:?}", label);
        }
    }
}
