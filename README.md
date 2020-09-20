rustc-worker
============

rustc-worker is an implementation of [Bazel Persistent
Workers](https://docs.bazel.build/versions/master/persistent-workers.html) for
Rust. It is meant to be used with
[rules_rust](https://github.com/bazelbuild/rules_rust). It can be used to speed
up building Rust code with Bazel by using a shared, incremental cache for
`rustc`.

In a default Bazel execution, each rustc compiler invocation is run in a
sandbox, which means that Bazel builds of Rust code only benefit from Bazel
caching at the crate boundaries. Each rebuild of a crate has to start from
scratch.

In worker mode, a thin wrapper around rustc creates a directory for rustc to
cache its [incremental compilation
artifacts](https://blog.rust-lang.org/2018/02/15/Rust-1.24.html), such that
rebuilding a crate can re-use unchanged parts of the crate.

This is _NOT_ a full persistent worker in the style of the
Java/TypeScript/Scala workers since `rustc` does not offer a "service" mode
where the same compiler process can accept multiple compilation requests and
re-use state like existing parse trees. There is a possibility that some of the
work from [rust-analyzer](https://rust-analyzer.github.io/) could enable such
behavior in the future.

On my Thinkpad x230, building [ninja-rs](https://github.com/nikhilm/ninja-rs),
here are the improvements I see when building the `ninja` binary, with a
comment-only change to `build/src/lib.rs`. (Using the `bazel` branch.)
All times are for debug builds as that is the standard developer workflow,
where incremental builds matter.

```
cargo build (incremental by default)  1.65s
bazel build (without worker)          2.47s
bazel build (with worker)             1.2s
```

Bazel is possibly slightly faster than Cargo due to not paying the cost of startup.

## How to use

This currently requires a special branch of `rules_rust` until it is accepted
and merged into the original rules.

Assuming you are already using `rules_rust`, you will need to make the
following changes to your `WORKSPACE` file.

1. Change your `rules_rust` repository to point to the branch, like this. This
   should replace any existing entry for the rules.

```
load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")
git_repository(
    name = "io_bazel_rules_rust",
    branch = "persistentworker",
    remote = "https://github.com/nikhilm/rules_rust",
)
```

2. Add a repository for the rustc-worker binary for your platform.

```
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

http_file(
    name = "rustc_worker",
    urls = ["https://github.com/nikhilm/rustc-worker/releases/download/v0.1.0/rustc-worker-linux"],
    sha256 = "0e2be6198d556e4972c52ee4bcd76be2c2d8fd74c58e0bf9be195c798e2a9a4e",
    executable = True,
)
```

That's it! Bazel 0.27 and higher will use workers by default when available. You can simply build any Rust targets as usual with Bazel.

If you want to play with this, but don't have an existing Rust project handy, you can:

```
git clone https://github.com/nikhilm/ninja-rs
cd ninja-rs
git checkout bazel
bazel build ninja
```

## TODO

[ ] Tests
[ ] Post on URLO and bazel-discuss whether sharing the incremental directory like this is even the right approach.
[ ] Create binary releases for other platforms, and automate this.
[ ] Submit issue for rules\_rust.
[ ] How to build with Bazel to bootstrap in rules\_rust.
[ ] Submit PR for rules\_rust.

## Contributing

Please file an issue discussing what you want to do if you are doing any major changes.
