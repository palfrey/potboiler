load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_rust",
    sha256 = "e6d835ee673f388aa5b62dc23d82db8fc76497e93fa47d8a4afe97abaf09b10d",
    strip_prefix = "rules_rust-f37b9d6a552e9412285e627f30cb124e709f4f7a",
    urls = [
        # Master branch as of 2021-01-27
        "https://github.com/bazelbuild/rules_rust/archive/f37b9d6a552e9412285e627f30cb124e709f4f7a.tar.gz",
    ],
)

load("@rules_rust//rust:repositories.bzl", "rust_repositories")
rust_repositories(version = "1.60.0", edition="2018")

http_archive(
    name = "io_bazel_rules_docker",
    sha256 = "a139e494d955fa133acb48bd7adc1a0b803139c0649f690c60b711700a24ec30",
    strip_prefix = "rules_docker-0.15.1-alpha",
    urls = ["https://github.com/dmayle/rules_docker/releases/download/v0.15.1-alpha/rules_docker-v0.15.1-alpha.tar.gz"],
)

load(
    "@io_bazel_rules_docker//repositories:repositories.bzl",
    container_repositories = "repositories",
)

container_repositories()

load(
    "@io_bazel_rules_docker//rust:image.bzl",
    _rust_image_repos = "repositories",
)

_rust_image_repos()

load("//bazel:crates.bzl", "raze_fetch_remote_crates")

raze_fetch_remote_crates()
