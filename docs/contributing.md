# Contributing to OasysDB

First of all, thank you for considering to contribute to OasysDB! We welcome
contributions from the community, and this documentation outlines the process to
start contributing to our project.

## Code of Conduct

We are committed to building an inclusive and welcoming community because we
believe that it will lead to a more successful project and a better experience
for everyone involved. To achieve that, any participant in our project is
expected to act respectfully and to follow the Code of Conduct.

## Have questions or suggestions?

[![Discord](https://img.shields.io/discord/1182432298382131200?logo=discord&logoColor=%23ffffff&label=Discord&labelColor=%235865F2&style=for-the-badge)][discord]

There is no such thing as a stupid question. If you have a question, chances
are, someone else does too. So, please feel free to ask questions whether it's
on our [Discord][discord] server or by opening a new discussion on [GitHub
Discussions][gh_discussions].

## Encounter a bug? Have a feature request?

If you encounter a bug or have a feature request, please open an issue on
[GitHub Issues][gh_issues]. Please include enough information for us to
understand the issue or the feature request. For this reason, we recommend you
to follow the issue templates we have provided when creating a new issue.

## Want to contribute code?

**TLDR: Check or open an issue first before working on a PR.**

Before you start working on a pull request, we encourage you to check out the
existing issues and pull requests to make sure that the feature you want to work
on is in our roadmap and is aligned with the project's vision. After all, we
don't want you to waste your time working on something that might not be merged.

We try to prioritize features and bug fixes that are on our roadmap or requested
a lot by the community. If you want to work on a feature or a fix that isn't
already in the issue tracker, please open an issue first to discuss it with the
project maintainers and the community.

For features, we try to prioritize features that are backed by real-world use
cases. If you have a use case for a feature, please include it in the issue.
We'd love to hear about it!

## Getting started

OasysDB is written in Rust. So, you need to have Rust installed on your local
machine. If you haven't installed Rust yet, you can install it by following the
instructions on the [Rust Installation Guide][rustup].

TODO: Complete the getting started guide.

## Style guide

We mostly use the default linting and style guide for Rust except for some
linting changes listed in the rustfmt.toml file. For more information about the
code style, see the [Rust Style Guide][style_guide].

For commit messages, we use the [Conventional Commits][conventional_commits]
format. This allows us to maintain consistency and readability in our Git commit
history making it easier to understand the changes made to the codebase at a
high-level.

When commenting your code, please try your best to write comments that are clear
and concise with proper English sentence capitalization and punctuation. This
will help us and the community understand your code better and keep the codebase
maintainable.

## Submitting a pull request

Once you have made your changes, you can submit a pull request. We will review
your pull request and provide feedback. If your pull request is accepted, we
will merge it into the main branch.

For organization purposes, we ask that you use the [Conventional
Commits][conventional_commits] format for your pull request title in lowercase:

```
<type>: <description>
```

For example:

```
feat: add support ...
fix: fix issue ...
```

## Conclusion

Thank you for taking the time to read this documentation. We look forward to
your contributions! Another way to support this project is to star this project,
share it with your circles, and join us on [Discord][discord].

Best regards,<br /> Edwin Kys

[discord]: https://discord.gg/bDhQrkqNP4
[gh_issues]: https://github.com/oasysai/oasysdb/issues
[gh_discussions]: https://github.com/oasysai/oasysdb/discussions
[rustup]: https://www.rust-lang.org/tools/install
[style_guide]: https://doc.rust-lang.org/beta/style-guide/index.html
[conventional_commits]: https://www.conventionalcommits.org/en/v1.0.0/
