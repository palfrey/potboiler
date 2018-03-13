git_repository(
    name = "io_bazel_rules_docker",
    remote = "https://github.com/bazelbuild/rules_docker.git",
    tag = "v0.4.0",
)

git_repository(
    name = "io_bazel_rules_rust",
    commit = "df95c3e3cd5afd87a69fa71dc9a56a0d0baa7823",
    remote = "https://github.com/bazelbuild/rules_rust.git",
)

load("@io_bazel_rules_rust//rust:repositories.bzl", "rust_repositories")

rust_repositories()

load(
    "@io_bazel_rules_docker//rust:image.bzl",
    _rust_image_repos = "repositories",
)

_rust_image_repos()

load("//cargo:crates.bzl", "raze_fetch_remote_crates")

raze_fetch_remote_crates()
