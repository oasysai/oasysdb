# Contributing to OasysDB

First of all, thank you for considering contributing to OasysDB! We welcome contributions from the community, and this document outlines the process for contributing to our project.

## Code of Conduct

We are committed to building an inclusive and welcoming community. We believe that it will lead to a more successful project and a better experience for everyone involved. To achieve that, any participant in our project is expected to act respectfully and to follow the [Code of Conduct](/docs/code_of_conduct.md).

## Have questions or suggestions?

[![Discord](https://img.shields.io/discord/1182432298382131200?logo=discord&logoColor=%23ffffff&label=Discord&labelColor=%235865F2&style=for-the-badge)](https://discord.gg/bDhQrkqNP4)

There is no such thing as a stupid question. If you have a question, chances are someone else does too. We encourage you to ask questions on our [Discord](https://discord.gg/bDhQrkqNP4) server. Alternatively, you can open a discussion on [GitHub Discussions](https://github.com/oasysai/oasysdb/discussions) with your question or suggestion.

## Encounter a bug? Have a feature request?

If you encounter a bug or have a feature request, please open an issue on [GitHub Issues](https://github.com/oasysai/oasysdb/issues). Please include as much information as possible in your issue. This includes:

- A description of the bug or feature request.
- If it's a bug, steps to reproduce the bug. If it's a feature request, include the use case and expected behavior of the feature.
- Screenshots or screen recording, if applicable.

## Want to contribute code?

**TLDR: Check and open an issue first before forking the repository and submitting a pull request.**

Before you start working on a pull request, we encourage you to check out the existing issues and pull requests to make sure that
the feature you want to work on is in our roadmap and is aligned with the project's vision. After all, we don't want you to waste your precious time!

We try to prioritize features and bug fixes that are on our roadmap or requested a lot by the community. If you want to work on a feature or bug fix that isn't already in the issue tracker, please open an issue first to discuss it with the community.

For features, we try to prioritize features that are backed by real-world use cases. If you have a use case for a feature, please include it in the issue. We'd love to hear about it!

# Getting started

Getting started with OasysDB development is easy.

You will need to have Rust installed. We recommend using [rustup](https://www.rust-lang.org/tools/install) to install Rust. We also recommend having rust-analyzer installed for your code editor.

After that, you need to install Maturin, which is a Python library used by OasysDB for building and publishing its Python packages. You can install Maturin using the following command:

```bash
pip install maturin
```

After setting up Maturin, fork the repository and clone it to your local machine. Then, in the root directory of the project, you need to set up and activate Python virtual environment for the project with `requirements.txt` as the dependency.

Once everything is set, you can run the following commands in the root directory of the repository:

```bash
# Run Rust tests.
cargo test

# Install OasysDB as a Python package.
maturin dev

# Run Python tests.
pytest
```

These commands will run the tests to make sure that everything is working as expected before you start working on your changes.

```bash
cargo bench
```

This command will run the benchmarks to measure the performance of the vector database. This is useful to make sure that your changes don't introduce any significant performance regressions.

## Style guide

We use mostly the default linting and style guide for Rust except for some linting changes listed in [rustfmt.toml](rustfmt.toml) file. For more information, see the [Rust Style Guide](https://doc.rust-lang.org/beta/style-guide/index.html).

For commit messages, we use the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format. This allows us to maintain consistency and readability in our commit messages.

When commenting your code, please try your best to write comments that are clear and concise with proper English sentence capitalization and punctuation. This will help us and the community understand your code better and keep the codebase maintainable.

## Submitting a pull request

Once you have made your changes, you can submit a pull request. We will review your pull request and provide feedback. If your pull request is accepted, we will merge it into the main branch.

For organization purposes, we ask that you use the following format for your pull request title in lowercase:

```
<type>: <description>
```

For example:

```
feat: add support ...
fix: fix issue ...
```

This is similar to the format used in [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

## Conclusion

Thank you for taking the time to read this documentation. We look forward to your contributions! Another way to support this project is to star this project, share it with your circles, and join us on [Discord](https://discord.gg/bDhQrkqNP4).

Best regards,<br />
Edwin Kys
