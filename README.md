# maddie.wtf

My personal website and blog, hosted at [maddie.wtf]. This is essentially a custom static site
generator, which server-side renders everything and serves it with [`axum`][axum].

A few slightly cool things it does:

- It watches for filesystem notifications in the content directory using [`notify`][notify] and
  hot-reloads any changed content. When running in debug mode, it also automatically reloads the
  page when this happens.
- Custom middleware intercepts `HandlerError`s returned from the request handlers and renders them
  with a template just like any other page, making the error handling code for each endpoint
  minimal.
- Commit info is gathered at build time so that the footer on every page can link back to the exact
  version that's being served.

This isn't really designed for anyone else to be able to come along and just _run_. That being said,
the licenses allow re-use, and feel free to take inspiration from anything I've done here. I am not
ever going to convert this project into a more general static site generator.

The real content is hosted in a separate private repo so I can write blog posts and commit them
before they're finished without them being accessible to the whole world. There's example content in
this repo, in `/example-content`, if you want to run this locally.

## Development

### Set up Rust toolchain

If you're using `rustup`, there's a `rust-toolchain.toml` file in the root of the repository that
specifies the toolchain needed to build the crate. To format the code, though, you'll have to
install a nightly toolchain and ensure it contains the `rustfmt` component, and then format with
`cargo +nightly fmt` or `rustup nightly run rustfmt`.

If you're using Nix, there's a flake that defines a dev shell with a Rust toolchain (including
nightly `rustfmt`) and a few other useful tools included. You can load this either with `nix
develop` or via the [`direnv`][direnv] `.envrc` file, and then build the crate as normal with `cargo
build`. Formatting should work out of the box with `cargo fmt` or `rustfmt`.

There's also a Nix package defined in the flake for release builds.

### Cutting a Release

This project uses [Conventional Commits][conventional-commits], and [`convco`][convco] is included
in the Nix devShell to assist with this.

The overall list of things that has to happen for each release is as follows:

- The commit that changes the version should use the message `release: v<version>`.
- That commit should update the version in `Cargo.toml` to match the version output by `convco
  version --bump`.
- That commit should include an updated `Cargo.lock` file, most easily generated by running a `cargo
  build` after updating the version.
- That commit should include an updated `Cargo.nix` file, generated using `cargo2nix -ls > Cargo.nix
  && nix fmt`.
- That commit should include the updated `CHANGELOG.md`, generated with `convco changelog -u
  $(convco version --bump) > CHANGELOG.md`.
  - Unfortunately, after doing this you'll have to update the file to replace `HEAD` in the URL in
    the new section heading with `v<version>`.
- That commit should be tagged with `v<version>`.
- The commit should be pushed to `main`, and the tag pushed as well. This will trigger the
  [`cargo-dist`][cargo-dist] workflow to build the artifacts and create the release on GitHub.

[axum]: https://github.com/tokio-rs/axum
[cargo-dist]: https://github.com/axodotdev/cargo-dist
[convco]: https://github.com/convco/convco
[conventional-commits]: https://www.conventionalcommits.org/en/v1.0.0/
[direnv]: https://github.com/direnv/direnv
[maddie.wtf]: https://maddie.wtf
[notify]: https://github.com/notify-rs/notify
