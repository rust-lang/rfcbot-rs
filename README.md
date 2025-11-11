# rfcbot

> [!NOTE]
> The infra team lost access to the old rfcbot account, i.e.
> [rfcbot](https://github.com/rfcbot).
>
> The new account used by the bot is [rust-rfcbot](https://github.com/rust-rfcbot).
>
> You still need to ping the old account, i.e. `@rfcbot`.

[rfcbot](https://github.com/rfcbot) manages asynchronous decision making on Rust issues and PRs. Status of Final Comment Periods can be viewed on [the relevant dashboard page](http://rfcbot.rs).

It listens for commands on all repositories owned by the [rust-lang](https://github.com/rust-lang), [rust-lang-nursery](https://github.com/rust-lang-nursery), and [rust-lang-deprecated](https://github.com/rust-lang-deprecated) organizations.

While its intended usage is for RFCs, you can use its tracking on any issue or pull request which needs an async review/decision cycle.

## Usage

rfcbot accepts commands in GitHub comments. All commands take the form:

```
@rfcbot COMMAND [PARAMS]
```

Each command must start on its own line, but otherwise the bot doesn't care if there's other text in the comment. This is valid:

```
TEXT
TEXT
@rfcbot fcp merge compiler
TEXT TEXT
TEXT
```

But this is not:

```
TEXT @rfcbot fcp merge compiler
TEXT
```

Both of these commands will be registered:

```
@rfcbot concern FOO
@rfcbot concern BAR
```

Examples are in each section.

### Command grammar

rfcbot accepts roughly the following grammar:

```ebnf
merge ::= "merge" | "merged" | "merging" | "merges" ;
close ::= "close" | "closed" | "closing" | "closes" ;
postpone ::= "postpone" | "postponed" | "postponing" | "postpones" ;
cancel ::= "cancel" | "canceled" | "canceling" | "cancels" ;
review ::= "reviewed" | "review" | "reviewing" | "reviews" ;
concern ::= "concern" | "concerned" | "concerning" | "concerns" ;
resolve ::= "resolve" | "resolved" | "resolving" | "resolves" ;
poll ::= "ask" | "asked" | "asking" | "asks" |
         "poll" | "polled" | "polling" | "polls" |
         "query" | "queried" | "querying" | "queries" |
         "inquire" | "inquired" | "inquiring" | "inquires" |
         "quiz" | "quizzed" | "quizzing" | "quizzes" |
         "survey" | "surveyed" | "surveying" | "surveys" ;

team_label ::= "T-lang" | .. ;
team_label_simple ::= "lang" | .. ;
team_label_any ::= team_label | team_label_simple ;
team_ping ::= "@"? "rust-lang/lang" | ..;
team_target ::= team_label | team_label_simple | team_ping ;
team_list ::= team_label_any (',' team_label_any)*

line_remainder ::= .+$ ;
ws_separated ::= ... ;

subcommand ::= merge [team_list]
             | close | postpone | cancel | review
             | concern line_remainder
             | resolve line_remainder
             | poll [team_target]* line_remainder
             ;

invocation ::= "fcp" subcommand
             | "pr" subcommand
             | "f?" ws_separated
             | subcommand
             ;

grammar ::= "@rfcbot" ":"? invocation ;
```

Multiple occurrences of `grammar` are allowed in each comment you make on GitHub.
This means that the following is OK:

```
@rfcbot merge compiler,lang

Some stuff you want to say...

@rfcbot concern foobar

Explain the concern...
```

### Final Comment Period

Before proposing a final comment period on an issue/PR/RFC, please double check to make sure that the correct team label(s) has been applied to the issue. Rfcbot recognizes the teams from the [teams API](https://team-api.infra.rust-lang.org/v1/rfcbot.json).

#### Proposing FCP

To propose an FCP, use `@rfcbot fcp merge [team(,team)*]`. You can close or postpone a proposed FCP using `@rfcbot fcp [close|postpone]`. You can also use `pr` instead of `fcp`, which will be used in the future to improve the quality of status comments from the bot. A team or list of teams must be specified if the issue is labeled with multiple teams, otherwise the labeled team will be used.

If the proposer is on one of the tagged subteams, rfcbot will create a tracking comment with a checklist of review requests. Once all review requests have been satisfied and any concerns have been resolved, it will post a comment to that effect. One week after the "FCP start" comment, it will post another follow-up comment saying that one week has passed.

rfcbot will only request reviews from members of the tagged team(s), and as of right now only supports reviews from teams that are tagged at the time an FCP is proposed.

#### Cancelling FCP

To cancel an FCP proposal after it's started, use `@rfcbot fcp cancel`. This will delete all records of the FCP, including any concerns raised (although their comments will remain).

#### Reviewing

To indicate that you've reviewed the FCP proposal, either check the box next to your name on the tracking comment, or use the command `@rfcbot reviewed`.

#### Concerns

To register blocking concerns on the FCP proposal, use `@rfcbot concern NAME_OF_CONCERN`. The bot will parse up until the first newline after the command for the concern's name, and add it to the list of concerns in the tracking comment.

To indicate that your concern has been resolved, use `@rfcbot resolved NAME_OF_CONCERN`. Note that as of this writing, only the original author can mark their concern as resolved.

Note that only one concern per comment is allowed.

### Feedback Requests

To request feedback from a user not on the tagged team(s), use `@rfcbot f? @username`. This will create an entry in the database which will be marked as resolved once that user has commented on the issue/PR. Note that these feedback requests will not block start/end of an FCP. If you need to block FCP on that user's feedback, you may want to create a new concern that you can resolve.

In a future update, the UI for the dashboard will be updated to display these feedback requests, but they don't show up anywhere right now.

## Contributing, Code of Conduct, License

Please see CONTRIBUTING.md.
