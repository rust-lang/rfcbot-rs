use std::collections::BTreeSet;
use std::fmt;

use error::{DashResult, DashError};
use config::RFC_BOT_MENTION;
use teams::{self, TeamLabel};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Label {
    FFCP,
    PFCP,
    FCP,
    Postponed,
    Closed,
    DispositionMerge,
    DispositionClose,
    DispositionPostpone,
}

impl Label {
    pub fn as_str(self) -> &'static str {
        use self::Label::*;
        match self {
            FFCP => "finished-final-comment-period",
            PFCP => "proposed-final-comment-period",
            FCP => "final-comment-period",
            Postponed => "postponed",
            Closed => "closed",
            DispositionMerge => "disposition-merge",
            DispositionClose => "disposition-close",
            DispositionPostpone => "disposition-postpone",
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.as_str())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RfcBotCommand<'a> {
    FcpPropose(FcpDisposition),
    FcpCancel,
    Reviewed,
    NewConcern(&'a str),
    ResolveConcern(&'a str),
    FeedbackRequest(&'a str),
    AskQuestion {
        teams: BTreeSet<&'a str>,
        question: &'a str,
    },
}

impl<'a> RfcBotCommand<'a> {
    pub fn from_str_all(command: &'a str) -> impl Iterator<Item = RfcBotCommand<'a>> {
        // Get the tokens for each command line (starts with a bot mention)
        command.lines()
               .filter(|&l| l.starts_with(RFC_BOT_MENTION))
               .map(from_invocation_line)
               .filter_map(Result::ok)
    }

}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FcpDisposition {
    Merge,
    Close,
    Postpone,
}

const FCP_REPR_MERGE: &'static str = "merge";
const FCP_REPR_CLOSE: &'static str = "close";
const FCP_REPR_POSTPONE: &'static str = "postpone";

impl FcpDisposition {
    pub fn repr(self) -> &'static str {
        match self {
            FcpDisposition::Merge => FCP_REPR_MERGE,
            FcpDisposition::Close => FCP_REPR_CLOSE,
            FcpDisposition::Postpone => FCP_REPR_POSTPONE,
        }
    }

    pub fn from_str(string: &str) -> DashResult<Self> {
        Ok(match string {
            FCP_REPR_MERGE => FcpDisposition::Merge,
            FCP_REPR_CLOSE => FcpDisposition::Close,
            FCP_REPR_POSTPONE => FcpDisposition::Postpone,
            _ => throw!(DashError::Misc(None)),
        })
    }

    pub fn label(self) -> Label {
        match self {
            FcpDisposition::Merge => Label::DispositionMerge,
            FcpDisposition::Close => Label::DispositionClose,
            FcpDisposition::Postpone => Label::DispositionPostpone,
        }
    }
}

/// Parses the text of a subcommand.
fn parse_command_text<'a>(command: &'a str, subcommand: &'a str) -> &'a str {
    let name_start = command.find(subcommand).unwrap() + subcommand.len();
    command[name_start..].trim()
}

fn strip_prefix<'h>(haystack: &'h str, prefix: &str) -> &'h str {
    haystack.find(prefix)
            .map(|idx| &haystack[idx + prefix.len()..])
            .unwrap_or(haystack)
            .trim()
}

fn match_team_candidate(team_candidate: &str) -> Option<&'static TeamLabel> {
    #[cfg(not(test))]
    let setup = &*teams::SETUP;
    #[cfg(test)]
    let setup = &*teams::test::TEST_SETUP;

    setup.teams().find(|&(label, team)| {
        strip_prefix(&label.0, "T-") == strip_prefix(team_candidate, "T-") ||
        team.ping() == strip_prefix(team_candidate, "@")
    }).map(|(label, _)| label)
}

/// Parses all subcommands under the fcp command.
/// If `fcp_context` is set to false, `@rfcbot <subcommand>`
/// was passed and not `@rfcbot fcp <subcommand>`.
/// 
/// @rfcbot accepts roughly the following grammar:
///
/// merge ::= "merge" | "merged" | "merging" | "merges" ;
/// close ::= "close" | "closed" | "closing" | "closes" ;
/// postpone ::= "postpone" | "postponed" | "postponing" | "postpones" ;
/// cancel ::= "cancel | "canceled" | "canceling" | "cancels" ;
/// review ::= "reviewed" | "review" | "reviewing" | "reviews" ;
/// concern ::= "concern" | "concerned" | "concerning" | "concerns" ;
/// resolve ::= "resolve" | "resolved" | "resolving" | "resolves" ;
/// poll ::=  "ask" | "asked" | "asking" | "asks" |
///          "poll" | "polled" | "polling" | "polls" |
///          "query" | "queried" | "querying" | "queries" |
///          "inquire" | "inquired" | "inquiring" | "inquires" |
///          "quiz" | "quized" | "quizing" | "quizzes" |
///          "survey" | "surveyed" | "surveying" | "surveys" ;
///
/// team_label ::= "T-lang" | .. ;
/// team_label_simple ::= "lang" | .. ;
/// team_ping ::= "@"? "rust-lang/lang" | ..;
/// team_target ::= team_label | team_label_simple | team_ping ;
///
/// line_remainder ::= .+$ ;
/// ws_separated ::= ... ;
///
/// subcommand ::= merge | close | postpone | cancel | review
///              | concern line_remainder
///              | resolve line_remainder
///              | poll [team_target]* line_remainder
///              ;
///
/// invocation ::= "fcp" subcommand
///              | "pr" subcommand
///              | "f?" ws_separated
///              | subcommand
///              ;
///
/// grammar ::= "@rfcbot" ":"? invocation ;
fn parse_fcp_subcommand<'a>(
    command: &'a str,
    subcommand: &'a str,
    fcp_context: bool
) -> DashResult<RfcBotCommand<'a>> {
    Ok(match subcommand {
        // Parse a FCP merge command:
        "merge" | "merged" | "merging" | "merges" =>
            RfcBotCommand::FcpPropose(FcpDisposition::Merge),

        // Parse a FCP close command:
        "close" | "closed" | "closing" | "closes" =>
            RfcBotCommand::FcpPropose(FcpDisposition::Close),

        // Parse a FCP postpone command:
        "postpone" | "postponed" | "postponing" | "postpones" =>
            RfcBotCommand::FcpPropose(FcpDisposition::Postpone),

        // Parse a FCP cancel command:
        "cancel" | "canceled" | "canceling" | "cancels" =>
            RfcBotCommand::FcpCancel,

        // Parse a FCP reviewed command:
        "reviewed" | "review" | "reviewing" | "reviews" =>
            RfcBotCommand::Reviewed,

        // Parse a FCP concern command:
        "concern" | "concerned" | "concerning" | "concerns" => {
            debug!("Parsed command as NewConcern");
            RfcBotCommand::NewConcern(parse_command_text(command, subcommand))
        },

        // Parse a FCP resolve command:
        "resolve" | "resolved" | "resolving" | "resolves" => {
            debug!("Parsed command as ResolveConcern");
            RfcBotCommand::ResolveConcern(parse_command_text(command, subcommand))
        },

        // Parse an AskQuestion command:
        "ask" | "asked" | "asking" | "asks" |
        "poll" | "polled" | "polling" | "polls" |
        "query" | "queried" | "querying" | "queries" |
        "inquire" | "inquired" | "inquiring" | "inquires" |
        "quiz" | "quized" | "quizing" | "quizzes" |
        "survey" | "surveyed" | "surveying" | "surveys" => {
            debug!("Parsed command as AskQuestion");

            let mut question = parse_command_text(command, subcommand);
            let mut teams = BTreeSet::new();
            while let Some(team_candidate) = question.split_whitespace().next() {
                if let Some(team) = match_team_candidate(team_candidate) {
                    question = parse_command_text(question, team_candidate);
                    teams.insert(&*team.0);
                } else {
                    break;
                }
            }
            RfcBotCommand::AskQuestion { teams, question }
        },

        _ => {
            throw!(DashError::Misc(if fcp_context {
                error!("unrecognized subcommand for fcp: {}", subcommand);
                Some(format!("found bad subcommand: {}", subcommand))
            } else {
                None
            }))
        }
    })
}

fn from_invocation_line<'a>(command: &'a str) -> DashResult<RfcBotCommand<'a>> {
    let mut tokens = command.trim_left_matches(RFC_BOT_MENTION)
                            .trim_left_matches(':')
                            .trim()
                            .split_whitespace();
    let invocation = tokens.next().ok_or(DashError::Misc(None))?;
    match invocation {
        "fcp" | "pr" => {
            let subcommand = tokens.next().ok_or(DashError::Misc(None))?;

            debug!("Parsed command as new FCP proposal");

            parse_fcp_subcommand(command, subcommand, true)
        }
        "f?" => {
            let user =
                tokens
                    .next()
                    .ok_or_else(|| DashError::Misc(Some("no user specified".to_string())))?;

            if user.is_empty() {
                throw!(DashError::Misc(Some("no user specified".to_string())));
            }

            Ok(RfcBotCommand::FeedbackRequest(&user[1..]))
        }
        _ => parse_fcp_subcommand(command, invocation, false),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn multiple_commands() {
let text = r#"
someothertext
@rfcbot: resolved CONCERN_NAME
somemoretext

somemoretext

@rfcbot: fcp cancel
foobar
@rfcbot concern foobar
"#;

        let cmd = RfcBotCommand::from_str_all(text).collect::<Vec<_>>();
        assert_eq!(cmd, vec![
            RfcBotCommand::ResolveConcern("CONCERN_NAME"),
            RfcBotCommand::FcpCancel,
            RfcBotCommand::NewConcern("foobar"),
        ]);
    }

    fn ensure_take_singleton<I: Iterator>(mut iter: I) -> I::Item {
        let singleton = iter.next().unwrap();
        assert!(iter.next().is_none());
        singleton
    }

    macro_rules! justification {
        () => { "\n\nSome justification here." };
    }

    macro_rules! some_text {
        ($important: expr) => {
            concat!(" ", $important, "
someothertext
somemoretext

somemoretext")
        };
    }

    macro_rules! test_from_str {
        ($test: ident, [$($cmd: expr),+], $message: expr, $expected: expr) => {
            test_from_str!($test, [$(concat!($cmd, $message)),+], $expected);
        };

        ($test: ident, [$($cmd: expr),+], $expected: expr) => {
            #[test]
            fn $test() {
                let expected = $expected;

                $({
                    let body = concat!("@rfcbot: ", $cmd);
                    let body_no_colon = concat!("@rfcbot ", $cmd);

                    let with_colon =
                        ensure_take_singleton(RfcBotCommand::from_str_all(body));

                    let without_colon =
                        ensure_take_singleton(RfcBotCommand::from_str_all(body_no_colon));

                    assert_eq!(with_colon, without_colon);
                    assert_eq!(with_colon, expected);
                })+
            }
        };
    }

    test_from_str!(success_fcp_reviewed,
        ["reviewed", "review", "reviewing", "reviews",
         "fcp reviewed", "fcp review", "fcp reviewing",
         "pr reviewed", "pr review", "pr reviewing"],
        RfcBotCommand::Reviewed);

    test_from_str!(success_fcp_merge,
        ["merge", "merged", "merging", "merges",
         "fcp merge", "fcp merged", "fcp merging", "fcp merges",
         "pr merge", "pr merged", "pr merging", "pr merges"],
        justification!(),
        RfcBotCommand::FcpPropose(FcpDisposition::Merge));

    test_from_str!(success_fcp_close,
        ["close", "closed", "closing", "closes",
         "fcp close", "fcp closed", "fcp closing", "fcp closes",
         "pr close", "pr closed", "pr closing", "pr closes"],
        justification!(),
        RfcBotCommand::FcpPropose(FcpDisposition::Close));

    test_from_str!(success_fcp_postpone,
        ["postpone", "postponed", "postponing", "postpones",
         "fcp postpone", "fcp postponed", "fcp postponing", "fcp postpones",
         "pr postpone", "pr postponed", "pr postponing", "pr postpones"],
        justification!(),
        RfcBotCommand::FcpPropose(FcpDisposition::Postpone));

    test_from_str!(success_fcp_cancel,
        ["cancel", "canceled", "canceling", "cancels",
         "fcp cancel", "fcp canceled", "fcp canceling", "fcp cancels",
         "pr cancel", "pr canceled", "pr canceling", "pr cancels"],
        justification!(),
        RfcBotCommand::FcpCancel);

    test_from_str!(success_concern,
        ["concern", "concerned", "concerning", "concerns",
         "fcp concern", "fcp concerned", "fcp concerning", "fcp concerns",
         "pr concern", "pr concerned", "pr concerning", "pr concerns"],
        some_text!("CONCERN_NAME"),
        RfcBotCommand::NewConcern("CONCERN_NAME"));

    test_from_str!(success_resolve,
        ["resolve", "resolved", "resolving", "resolves",
         "fcp resolve", "fcp resolved", "fcp resolving", "fcp resolves",
         "pr resolve", "pr resolved", "pr resolving", "pr resolves"],
        some_text!("CONCERN_NAME"),
        RfcBotCommand::ResolveConcern("CONCERN_NAME"));

    test_from_str!(success_ask_question,
        ["ask", "asked", "asking", "asks",
         "poll", "polled", "polling", "polls",
         "query", "queried", "querying", "queries",
         "inquire", "inquired", "inquiring", "inquires",
         "quiz", "quized", "quizing", "quizzes",
         "survey", "surveyed", "surveying", "surveys",
         "fcp ask", "fcp asked", "fcp asking", "fcp asks",
         "fcp poll", "fcp polled", "fcp polling", "fcp polls",
         "fcp query", "fcp queried", "fcp querying", "fcp queries",
         "fcp inquire", "fcp inquired", "fcp inquiring", "fcp inquires",
         "fcp quiz", "fcp quized", "fcp quizing", "fcp quizzes",
         "fcp survey", "fcp surveyed", "fcp surveying", "fcp surveys",
         "pr ask", "pr asked", "pr asking", "pr asks",
         "pr poll", "pr polled", "pr polling", "pr polls",
         "pr query", "pr queried", "pr querying", "pr queries",
         "pr inquire", "pr inquired", "pr inquiring", "pr inquires",
         "pr quiz", "pr quized", "pr quizing", "pr quizzes",
         "pr survey", "pr surveyed", "pr surveying", "pr surveys"],
        some_text!("avengers T-justice-league TO BE OR NOT TO BE?"),
        RfcBotCommand::AskQuestion {
            teams: btreeset! {
                "T-avengers",
                "justice-league",
            },
            question: "TO BE OR NOT TO BE?",
        });

    #[test]
    fn success_resolve_mid_body() {
        let body = "someothertext
@rfcbot: resolved CONCERN_NAME
somemoretext

somemoretext";
        let body_no_colon = "someothertext
somemoretext

@rfcbot resolved CONCERN_NAME

somemoretext";

        let with_colon = ensure_take_singleton(RfcBotCommand::from_str_all(body));
        let without_colon =
            ensure_take_singleton(RfcBotCommand::from_str_all(body_no_colon));

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::ResolveConcern("CONCERN_NAME"));
    }

    test_from_str!(success_feedback, ["f?"], some_text!("@bob"),
        RfcBotCommand::FeedbackRequest("bob"));
}
