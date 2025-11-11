use std::collections::BTreeSet;
use std::fmt;

use crate::config::RFC_BOT_MENTION;
use crate::error::{DashError, DashResult};
use crate::teams::{RfcbotConfig, TeamLabel};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Label {
    FFCP,
    PFCP,
    FCP,
    Postponed,
    Closed,
    ToAnnounce,
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
            ToAnnounce => "to-announce",
            DispositionMerge => "disposition-merge",
            DispositionClose => "disposition-close",
            DispositionPostpone => "disposition-postpone",
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result { fmt.write_str(self.as_str()) }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FcpDisposition {
    Merge,
    Close,
    Postpone,
}

const FCP_REPR_MERGE: &str = "merge";
const FCP_REPR_CLOSE: &str = "close";
const FCP_REPR_POSTPONE: &str = "postpone";

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FcpDispositionData<'a> {
    Merge(BTreeSet<&'a str>),
    Close,
    Postpone,
}

impl FcpDispositionData<'_> {
    pub fn disp(&self) -> FcpDisposition {
        match self {
            FcpDispositionData::Merge(..) => FcpDisposition::Merge,
            FcpDispositionData::Close => FcpDisposition::Close,
            FcpDispositionData::Postpone => FcpDisposition::Postpone,
        }
    }
}

/// Parses the text of a subcommand.
fn parse_command_text<'a>(command: &'a str, subcommand: &'a str) -> &'a str {
    let name_start = command.find(subcommand).unwrap() + subcommand.len();
    command[name_start..].trim()
}

fn strip_prefix<'h>(haystack: &'h str, prefix: &str) -> &'h str {
    haystack
        .find(prefix)
        .map(|idx| &haystack[idx + prefix.len()..])
        .unwrap_or(haystack)
        .trim()
}

fn match_team_candidate<'a>(
    setup: &'a RfcbotConfig,
    team_candidate: &str,
) -> Option<&'a TeamLabel> {
    setup
        .teams()
        .find(|&(label, team)| {
            strip_prefix(&label.0, "T-") == strip_prefix(team_candidate, "T-")
                || team.ping() == strip_prefix(team_candidate, "@")
        })
        .map(|(label, _)| label)
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
///          "quiz" | "quizzed" | "quizzing" | "quizzes" |
///          "survey" | "surveyed" | "surveying" | "surveys" ;
///
/// team_label ::= "T-lang" | .. ;
/// team_label_simple ::= "lang" | .. ;
/// team_label_any ::= team_label | team_label_simple ;
/// team_ping ::= "@"? "rust-lang/lang" | ..;
/// team_target ::= team_label | team_label_simple | team_ping ;
/// team_list ::= team_label_any (',' team_label_any)*
///
/// line_remainder ::= .+$ ;
/// ws_separated ::= ... ;
///
/// subcommand ::= merge team_list
///              | close | postpone | cancel | review
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
    setup: &'a RfcbotConfig,
    command: &'a str,
    subcommand: &'a str,
    fcp_context: bool,
) -> DashResult<RfcBotCommand<'a>> {
    Ok(match subcommand {
        // Parse a FCP merge command:
        "merge" | "merged" | "merging" | "merges" => {
            debug!("Parsed command as FcpPropose(Merge(..))");

            let team_text = parse_command_text(command, subcommand);
            let mut teams = BTreeSet::new();
            for team_candidate in team_text.split(",") {
                if let Some(team) = match_team_candidate(setup, team_candidate) {
                    teams.insert(&*team.0);
                }
            }

            RfcBotCommand::FcpPropose(FcpDispositionData::Merge(teams))
        }

        // Parse a FCP close command:
        "close" | "closed" | "closing" | "closes" => {
            RfcBotCommand::FcpPropose(FcpDispositionData::Close)
        }

        // Parse a FCP postpone command:
        "postpone" | "postponed" | "postponing" | "postpones" => {
            RfcBotCommand::FcpPropose(FcpDispositionData::Postpone)
        }

        // Parse a FCP cancel command:
        "cancel" | "canceled" | "canceling" | "cancels" => RfcBotCommand::FcpCancel,

        // Parse a FCP reviewed command:
        "reviewed" | "review" | "reviewing" | "reviews" => RfcBotCommand::Reviewed,

        // Parse a FCP concern command:
        "concern" | "concerned" | "concerning" | "concerns" => {
            debug!("Parsed command as NewConcern");
            RfcBotCommand::NewConcern(parse_command_text(command, subcommand))
        }

        // Parse a FCP resolve command:
        "resolve" | "resolved" | "resolving" | "resolves" => {
            debug!("Parsed command as ResolveConcern");
            RfcBotCommand::ResolveConcern(parse_command_text(command, subcommand))
        }

        // Parse a StartPoll command:
        "ask" | "asked" | "asking" | "asks" | "poll" | "polled" | "polling" | "polls" | "query"
        | "queried" | "querying" | "queries" | "inquire" | "inquired" | "inquiring"
        | "inquires" | "quiz" | "quizzed" | "quizzing" | "quizzes" | "survey" | "surveyed"
        | "surveying" | "surveys" => {
            debug!("Parsed command as StartPoll");

            let mut question = parse_command_text(command, subcommand);
            let mut teams = BTreeSet::new();
            while let Some(team_candidate) = question.split_whitespace().next() {
                if let Some(team) = match_team_candidate(setup, team_candidate) {
                    question = parse_command_text(question, team_candidate);
                    teams.insert(&*team.0);
                } else {
                    break;
                }
            }
            RfcBotCommand::StartPoll { teams, question }
        }

        _ => throw!(DashError::Misc(if fcp_context {
            error!("unrecognized subcommand for fcp: {}", subcommand);
            Some(format!("found bad subcommand: {}", subcommand))
        } else {
            None
        })),
    })
}

fn from_invocation_line<'a>(
    setup: &'a RfcbotConfig,
    command: &'a str,
) -> DashResult<RfcBotCommand<'a>> {
    let mut tokens = command
        .trim_start_matches(RFC_BOT_MENTION)
        .trim()
        .trim_start_matches(':')
        .trim()
        .split_whitespace();
    let invocation = tokens.next().ok_or(DashError::Misc(None))?;
    match invocation {
        "fcp" | "pr" => {
            let subcommand = tokens.next().ok_or(DashError::Misc(None))?;

            debug!("Parsed command as new FCP proposal");

            parse_fcp_subcommand(setup, command, subcommand, true)
        }
        "f?" => {
            let user = tokens
                .next()
                .ok_or_else(|| DashError::Misc(Some("no user specified".to_string())))?;

            if user.is_empty() {
                throw!(DashError::Misc(Some("no user specified".to_string())));
            }

            Ok(RfcBotCommand::FeedbackRequest(&user[1..]))
        }
        _ => parse_fcp_subcommand(setup, command, invocation, false),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum RfcBotCommand<'a> {
    FcpPropose(FcpDispositionData<'a>),
    FcpCancel,
    Reviewed,
    NewConcern(&'a str),
    ResolveConcern(&'a str),
    FeedbackRequest(&'a str),
    StartPoll {
        teams: BTreeSet<&'a str>,
        question: &'a str,
    },
}

impl<'a> RfcBotCommand<'a> {
    pub fn from_str_all(
        setup: &'a RfcbotConfig,
        command: &'a str,
    ) -> impl Iterator<Item = RfcBotCommand<'a>> {
        // Get the tokens for each command line (starts with a bot mention)
        command
            .lines()
            .map(str::trim)
            .filter(|&l| l.starts_with(RFC_BOT_MENTION))
            .map(move |l| from_invocation_line(setup, l))
            .filter_map(Result::ok)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::teams::test::TEST_SETUP;

    fn parse_commands(body: &str) -> impl Iterator<Item = RfcBotCommand<'_>> {
        RfcBotCommand::from_str_all(&TEST_SETUP, body)
    }

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

        assert_eq!(
            parse_commands(text).collect::<Vec<_>>(),
            vec![
                RfcBotCommand::ResolveConcern("CONCERN_NAME"),
                RfcBotCommand::FcpCancel,
                RfcBotCommand::NewConcern("foobar"),
            ]
        );
    }

    #[test]
    fn accept_leading_whitespace() {
        let text = r#"
someothertext
       @rfcbot: resolved CONCERN_NAME
somemoretext

somemoretext

   @rfcbot: fcp cancel
foobar
 @rfcbot concern foobar
"#;

        assert_eq!(
            parse_commands(text).collect::<Vec<_>>(),
            vec![
                RfcBotCommand::ResolveConcern("CONCERN_NAME"),
                RfcBotCommand::FcpCancel,
                RfcBotCommand::NewConcern("foobar"),
            ]
        );
    }

    #[test]
    fn fix_issue_225() {
        let text = r#"
someothertext
    @rfcbot : resolved CONCERN_NAME
somemoretext

somemoretext

@rfcbot : fcp cancel
foobar
@rfcbot : concern foobar
"#;

        assert_eq!(
            parse_commands(text).collect::<Vec<_>>(),
            vec![
                RfcBotCommand::ResolveConcern("CONCERN_NAME"),
                RfcBotCommand::FcpCancel,
                RfcBotCommand::NewConcern("foobar"),
            ]
        );
    }

    fn ensure_take_singleton<I: Iterator>(mut iter: I) -> I::Item {
        let singleton = iter.next().unwrap();
        assert!(iter.next().is_none());
        singleton
    }

    macro_rules! justification {
        () => {
            "\n\nSome justification here."
        };
    }

    macro_rules! some_text {
        ($important: expr) => {
            concat!(
                " ",
                $important,
                "
someothertext
somemoretext

somemoretext"
            )
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

                    let with_colon = ensure_take_singleton(parse_commands(body));
                    let without_colon = ensure_take_singleton(parse_commands(body_no_colon));

                    assert_eq!(with_colon, without_colon);
                    assert_eq!(with_colon, expected);
                })+
            }
        };
    }

    test_from_str!(
        success_fcp_reviewed,
        [
            "reviewed",
            "review",
            "reviewing",
            "reviews",
            "fcp reviewed",
            "fcp review",
            "fcp reviewing",
            "pr reviewed",
            "pr review",
            "pr reviewing"
        ],
        RfcBotCommand::Reviewed
    );

    test_from_str!(
        success_fcp_merge,
        [
            "merge",
            "merged",
            "merging",
            "merges",
            "fcp merge",
            "fcp merged",
            "fcp merging",
            "fcp merges",
            "pr merge",
            "pr merged",
            "pr merging",
            "pr merges"
        ],
        justification!(),
        RfcBotCommand::FcpPropose(FcpDispositionData::Merge(BTreeSet::new()))
    );

    test_from_str!(
        success_fcp_merge_teams,
        [
            "merge compiler,lang",
            "merged compiler,lang",
            "merging compiler,lang",
            "merges compiler,lang",
            "fcp merge compiler,lang",
            "fcp merged compiler,lang",
            "fcp merging compiler,lang",
            "fcp merges compiler,lang",
            "pr merge compiler,lang",
            "pr merged compiler,lang",
            "pr merging compiler,lang",
            "pr merges compiler,lang"
        ],
        justification!(),
        RfcBotCommand::FcpPropose(FcpDispositionData::Merge(["compiler", "lang"].iter().copied().collect()))
    );

    test_from_str!(
        success_fcp_close,
        [
            "close",
            "closed",
            "closing",
            "closes",
            "fcp close",
            "fcp closed",
            "fcp closing",
            "fcp closes",
            "pr close",
            "pr closed",
            "pr closing",
            "pr closes"
        ],
        justification!(),
        RfcBotCommand::FcpPropose(FcpDispositionData::Close)
    );

    test_from_str!(
        success_fcp_postpone,
        [
            "postpone",
            "postponed",
            "postponing",
            "postpones",
            "fcp postpone",
            "fcp postponed",
            "fcp postponing",
            "fcp postpones",
            "pr postpone",
            "pr postponed",
            "pr postponing",
            "pr postpones"
        ],
        justification!(),
        RfcBotCommand::FcpPropose(FcpDispositionData::Postpone)
    );

    test_from_str!(
        success_fcp_cancel,
        [
            "cancel",
            "canceled",
            "canceling",
            "cancels",
            "fcp cancel",
            "fcp canceled",
            "fcp canceling",
            "fcp cancels",
            "pr cancel",
            "pr canceled",
            "pr canceling",
            "pr cancels"
        ],
        justification!(),
        RfcBotCommand::FcpCancel
    );

    test_from_str!(
        success_concern,
        [
            "concern",
            "concerned",
            "concerning",
            "concerns",
            "fcp concern",
            "fcp concerned",
            "fcp concerning",
            "fcp concerns",
            "pr concern",
            "pr concerned",
            "pr concerning",
            "pr concerns"
        ],
        some_text!("CONCERN_NAME"),
        RfcBotCommand::NewConcern("CONCERN_NAME")
    );

    test_from_str!(
        success_resolve,
        [
            "resolve",
            "resolved",
            "resolving",
            "resolves",
            "fcp resolve",
            "fcp resolved",
            "fcp resolving",
            "fcp resolves",
            "pr resolve",
            "pr resolved",
            "pr resolving",
            "pr resolves"
        ],
        some_text!("CONCERN_NAME"),
        RfcBotCommand::ResolveConcern("CONCERN_NAME")
    );

    test_from_str!(
        success_ask_question,
        [
            "ask",
            "asked",
            "asking",
            "asks",
            "poll",
            "polled",
            "polling",
            "polls",
            "query",
            "queried",
            "querying",
            "queries",
            "inquire",
            "inquired",
            "inquiring",
            "inquires",
            "quiz",
            "quizzed",
            "quizzing",
            "quizzes",
            "survey",
            "surveyed",
            "surveying",
            "surveys",
            "fcp ask",
            "fcp asked",
            "fcp asking",
            "fcp asks",
            "fcp poll",
            "fcp polled",
            "fcp polling",
            "fcp polls",
            "fcp query",
            "fcp queried",
            "fcp querying",
            "fcp queries",
            "fcp inquire",
            "fcp inquired",
            "fcp inquiring",
            "fcp inquires",
            "fcp quiz",
            "fcp quizzed",
            "fcp quizzing",
            "fcp quizzes",
            "fcp survey",
            "fcp surveyed",
            "fcp surveying",
            "fcp surveys",
            "pr ask",
            "pr asked",
            "pr asking",
            "pr asks",
            "pr poll",
            "pr polled",
            "pr polling",
            "pr polls",
            "pr query",
            "pr queried",
            "pr querying",
            "pr queries",
            "pr inquire",
            "pr inquired",
            "pr inquiring",
            "pr inquires",
            "pr quiz",
            "pr quizzed",
            "pr quizzing",
            "pr quizzes",
            "pr survey",
            "pr surveyed",
            "pr surveying",
            "pr surveys"
        ],
        some_text!("avengers T-justice-league TO BE OR NOT TO BE?"),
        RfcBotCommand::StartPoll {
            teams: btreeset! {
                "T-avengers",
                "justice-league",
            },
            question: "TO BE OR NOT TO BE?",
        }
    );

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

        let with_colon = ensure_take_singleton(parse_commands(body));
        let without_colon = ensure_take_singleton(parse_commands(body_no_colon));

        assert_eq!(with_colon, without_colon);
        assert_eq!(with_colon, RfcBotCommand::ResolveConcern("CONCERN_NAME"));
    }

    test_from_str!(
        success_feedback,
        ["f?"],
        some_text!("@bob"),
        RfcBotCommand::FeedbackRequest("bob")
    );
}
