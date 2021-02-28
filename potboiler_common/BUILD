package(default_visibility = ["//visibility:public"])

load("@io_bazel_rules_docker//rust:image.bzl", "rust_image")
load("@rules_rust//rust:rust.bzl", "rust_library")

rust_library(
    name = "potboiler_common",
    srcs = glob(["src/*.rs", "src/test/*.rs"]),
    deps = [
        "//potboiler_common/bazel:uuid",
        "//potboiler_common/bazel:postgres",
        "//potboiler_common/bazel:postgres_shared",
        "//potboiler_common/bazel:r2d2",
        "//potboiler_common/bazel:r2d2_postgres",
        "//potboiler_common/bazel:serde",
        "//potboiler_common/bazel:serde_json",
        "//potboiler_common/bazel:hybrid_clocks",
        "//potboiler_common/bazel:anyhow",
        "//potboiler_common/bazel:thiserror",
        "//potboiler_common/bazel:futures",
        "//potboiler_common/bazel:log",
        "//potboiler_common/bazel:actix",
        "//potboiler_common/bazel:regex",
        "//potboiler_common/bazel:actix_web",
        "//potboiler_common/bazel:actix_net",
    ],
    proc_macro_deps = [
        "//potboiler_common/bazel:serde_derive",
    ]
)