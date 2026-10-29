# bitkit-rs

[![Build status](https://github.com/bitshift-systems/bitkit-rs/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/bitshift-systems/bitkit-rs/actions/workflows/ci.yaml)

A service building kit written in Rust to help build software faster.

## Local documentation

Build a crate's rustdoc with all features enabled and open it in your browser:

```sh
mise run docs uhoh
mise run docs uhoh-macros
```

`docs` is an alias for `docs:rust`. Use the Cargo package name as the argument.
Dependency documentation is omitted.

## Releases

`uhoh` and `uhoh-macros` share a version and a GitHub release. Conventional
commits under `uhoh/` drive release-please's release PR. Merge that PR to
create a component tag (for example, `uhoh-0.1.0`) and trigger `release.yaml`.
The workflow matches the tag to a component in `release-please-config.json`
and discovers its publishable workspace crates using Cargo metadata. It
checks their versions, tests them, and publishes them in dependency order.
Crates belonging to other components are not published.

To add a component, add its package directory and explicit `component` name
to `release-please-config.json`, including any version updates needed for
its companion crates and workspace dependencies. Publishable crates beneath
that directory belong to the component. Keep independently released components
in separate, non-overlapping directories. No workflow changes are needed.

Configure these GitHub Actions values before the first release:

- Variable `BOT_CLIENT_ID` and secret `BOT_PRIVATE_KEY` for the release GitHub
  App, installed on this repository with write access to contents, issues,
  and pull requests. Its token allows the release event to trigger publishing.
- Secret `CARGO_REGISTRY_TOKEN` with permission to publish both `uhoh` and
  `uhoh-macros` on crates.io.

Release-please updates both package versions, the workspace dependency
versions, and the root lockfile together. Commit `Cargo.lock` along with the
manifests so the release workflow can use `--locked`.

To check the packages locally without uploading them:

```sh
cargo publish --package uhoh-macros --package uhoh --all-features --locked --registry crates-io --dry-run
```
