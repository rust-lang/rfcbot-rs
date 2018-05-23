# rfcbot

[rfcbot](https://github.com/rfcbot) manages asynchronous decision making on Rust issues and PRs. Status of Final Comment Periods can be viewed on [the relevant dashboard page](http://rfcbot.rs).

It listens for commands on all repositories owned by the [rust-lang](https://github.com/rust-lang), [rust-lang-nursery](https://github.com/rust-lang-nursery), and [rust-lang-deprecated](https://github.com/rust-lang-deprecated) organizations.

While its intended usage is for RFCs, you can use its tracking on any issue or pull request which needs an async review/decision cycle.

## Usage

rfcbot accepts commands in GitHub comments. All commands take the form:

```
@rfcbot COMMAND [PARAMS]
```

The bot expects one command per comment, and the command must start on its own line, but otherwise the bot doesn't care if there's other text in the comment. This is valid:

```
TEXT
TEXT
@rfcbot fcp merge
TEXT TEXT
TEXT
```

But this is not:

```
TEXT @rfcbot fcp merge
TEXT
```

Only the first of these commands will be registered:

```
@rfcbot concern FOO
@rfcbot concern BAR
```

Examples are in each section.

### Final Comment Period

Before proposing a final comment period on an issue/PR/RFC, please double check to make sure that the correct team label(s) has been applied to the issue. As of 9/17/16, rfcbot recognizes these labels:

* Core: `T-core`
* Language: `T-lang`
* Libraries: `T-libs`
* Compiler: `T-compiler`
* Tools: `T-tools`
* Documentation: `T-doc`

#### Proposing FCP

To propose an FCP, use `@rfcbot fcp DISPOSITION` where disposition is one of `[merge|close|postpone]`. You can also use `@rfcbot pr DISPOSITION`, which will be used in the future to improve the quality of status comments from the bot.

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
