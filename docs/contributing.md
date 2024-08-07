# Contributing to OasysDB

First of all, thank you for considering contributing to OasysDB! We welcome
contributions from the community, and this document outlines the process for
contributing to our project.

## Code of Conduct

We are committed to building an inclusive and welcoming community. We believe
that it will lead to a more successful project and a better experience for
everyone involved. To achieve that, any participant in our project is expected
to act respectfully and to follow the Code of Conduct.

## Have questions or suggestions?

[![Discord](https://img.shields.io/discord/1182432298382131200?logo=discord&logoColor=%23ffffff&label=Discord&labelColor=%235865F2&style=for-the-badge)][discord]

There is no such thing as a stupid question. If you have a question, chances are
someone else does too. We encourage you to ask questions on our
[Discord][discord] server. Alternatively, you can open a discussion on [GitHub
Discussions][gh_discussions] with your question or suggestion.

## Encounter a bug? Have a feature request?

If you encounter a bug or have a feature request, please open an issue on
[GitHub Issues][gh_issues]. Please include as much information as possible in
your issue. This includes:

- A description of the bug or feature request.
- If it's a bug, steps to reproduce the bug. If it's a feature request, include
  the use case and expected behavior of the feature.
- Screenshots or screen recording, if applicable.

## Want to contribute code?

**TLDR: Check and open an issue first before forking the repository and
submitting a pull request.**

Before you start working on a pull request, we encourage you to check out the
existing issues and pull requests to make sure that the feature you want to work
on is in our roadmap and is aligned with the project's vision. After all, we
don't want you to waste your precious time!

We try to prioritize features and bug fixes that are on our roadmap or requested
a lot by the community. If you want to work on a feature or bug fix that isn't
already in the issue tracker, please open an issue first to discuss it with the
community.

For features, we try to prioritize features that are backed by real-world use
cases. If you have a use case for a feature, please include it in the issue.
We'd love to hear about it!

## Getting started

Getting started with OasysDB development is pretty straightforward.

First, you will need to have Rust installed on your machine. We recommend using
[rustup][rustup] to install Rust. We also recommend having rust-analyzer
installed for your code editor for a better development experience.

All of the functionalities of OasysDB are available in the **src** directory.
The 2 most important modules are **db** and **indices** which respectively
contain the database functionalities and the index implementations.
Additionally, some custom types used throughout the project are defined in the
**types** module.

Before you start working on the code, I recommend you to run the tests to make
sure everything is working as expected. You can run the tests with the following
command:

```sh
cargo test
```

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
