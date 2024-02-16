# CosmWasm Minus

[![CircleCI](https://circleci.com/gh/CosmWasm/cw-minus/tree/main.svg?style=shield)](https://circleci.com/gh/CosmWasm/cw-minus/tree/main)

| Utilities      | Crates.io                                                                                                                     | Docs                                                                                | Coverage                                                                                                                                  |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| cw-utils           | [![cw-utils on crates.io](https://img.shields.io/crates/v/cw-utils.svg)](https://crates.io/crates/cw-utils)    | [![Docs](https://docs.rs/cw-utils/badge.svg)](https://docs.rs/cw-utils)   | [![codecov](https://codecov.io/gh/CosmWasm/cw-utils/branch/main/graph/badge.svg?token=IYY72ZVS3X)](https://codecov.io/gh/CosmWasm/cw-minus) |
| cw-controllers | [![cw-controllers on crates.io](https://img.shields.io/crates/v/cw-controllers.svg)](https://crates.io/crates/cw-controllers) | [![Docs](https://docs.rs/cw-controllers/badge.svg)](https://docs.rs/cw-controllers) | [![codecov](https://codecov.io/gh/CosmWasm/cw-minus/branch/main/graph/badge.svg?token=IYY72ZVS3X)](https://codecov.io/gh/CosmWasm/cw-minus) |
| cw2           | [![cw2 on crates.io](https://img.shields.io/crates/v/cw2.svg)](https://crates.io/crates/cw2)    | [![Docs](https://docs.rs/cw2/badge.svg)](https://docs.rs/cw2)   | [![codecov](https://codecov.io/gh/CosmWasm/cw-minus/branch/main/graph/badge.svg?token=IYY72ZVS3X)](https://codecov.io/gh/CosmWasm/cw-minus) |


Note: `cw2` and `controllers` have been moved from the [`cw-plus` repo](https://github.com/CosmWasm/cw-plus). Their commit history and changelog can be found in the previous repository.


## Generating changelog

To generate a changelog we decided to use
[github-changelog-generator](https://github.com/github-changelog-generator/github-changelog-generator).

To install tool you need Ruby's `gem` package manager.

    $ gem --user install github_changelog_generator

And put `$HOME/.gem/ruby/*/bin/` into your PATH.

Generating changelog file first time:

    $ github_changelog_generator -u CosmWasm -p cw-plus

Appending next releases could be done adding `--base` flag:

    $ github_changelog_generator -u CosmWasm -p cw-plus --base CHANGELOG.md

If you hit GitHub's 50 requests/hour limit, please follow
[this](https://github.com/github-changelog-generator/github-changelog-generator#github-token) guide to create a token
key which you can pass using `--token` flag.

There's also a convenience `scripts/update_changelog.sh`, which can take a --since-tag parameter (to avoid processing
the entire history). It can also auto-detect the latest version tag for you, with --latest-tag.

## Licenses

This repo is licensed under [Apache 2.0](./LICENSE).
