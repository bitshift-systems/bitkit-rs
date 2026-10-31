# BitKit

[![Build status](https://github.com/bitshift-systems/bitkit-rs/actions/workflows/ci.yaml/badge.svg?branch=main)](https://github.com/bitshift-systems/bitkit-rs/actions/workflows/ci.yaml)

A suite of crates written by Bitshift to help build software faster.

## Crates

These are the crates built within this repo:

| Crate            | Description                                            | Version                                                                             |
| ---------------- | ------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| [`uhoh`](./uhoh) | Declarative enum errors with HTTP integration          | [![Crates.io](https://img.shields.io/crates/v/uhoh)](https://crates.io/crates/uhoh) |
| [`olly`](./olly) | Observability (o11y) presets for Bitshift applications | [![Crates.io](https://img.shields.io/crates/v/olly)](https://crates.io/crates/olly) |

## About this Repo

The following sections explain the overall engineering process we use in this
repo.

### Adding new Crates

Adding a new crate to this repo is done by running the `rust:new` Mise task. It
will prompt you for a few bits of state, and then will wire everything up that
needs to be.

```
mise run rust:new --name=my-new-crate
```

### Publishing Crates

Releasing new versions of crates to crates.io is handled by CI. To release a new
version of any crate, follow this simple process:

1. Merge your changes to the crate into `main` via a Pull Request.
2. Wait for the [Release Please](https://github.com/bitshift-systems/bitkit-rs/actions/workflows/release-please.yaml)
   workflow to complete.
3. Review and merge the relevant release PR from @bitshift-releaser[bot].
4. The PR being merged will trigger a new release to get created, and the
   [Release](https://github.com/bitshift-systems/bitkit-rs/actions/workflows/release.yaml)
   workflow will run once per crate released.

Once the Release workflow is completed, the affected crates should be updated
on crates.io; docs.rs publishing may take some time to complete, so just be
patient for rustdocs.

> [!TIP]
> Crates with sub-crates (such as `uhoh` with `uhoh-macros`) stored in their
> respective directories will get versioned and released together. If you need
> to separate the release flows of sibling crates, don't nest their directories.

### Documenting Crates

We want our crates to have useful, helpful documentation such that consumers
within and without Bitshift can enjoy discovering and working with our features.
We expect our Rust code to be fully documented in the standard way for other
Rust projects (via Rustdoc), so all documentation should be handled by doc
comments in crates and READMEs that get attached to the relevant crates.

To generate and open documentation locally with all features enabled, run:

```bash
mise run docs:rust <crate>
```

For example:

```bash
# open the documentation for the `uhoh` crate
mise run docs:rust uhoh
```

## License

This project (and all its crates) is provided under the [Apache 2.0](./LICENSE)
license.
