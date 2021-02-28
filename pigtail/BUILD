load("@io_bazel_rules_docker//rust:image.bzl", "rust_image")
load("@rules_rust//rust:rust.bzl", "rust_binary", "rust_library")

rust_library(
    name = "pigtail_lib",
    srcs = glob(["src/*.rs"], exclude=["src/main.rs"]),
    deps = [
        "//pigtail/bazel:log4rs",
        "//pigtail/bazel:serde",
        "//pigtail/bazel:serde_json",
        "//pigtail/bazel:url",
        "//pigtail/bazel:hybrid_clocks",
        "//potboiler_common:potboiler_common",
        "//pigtail/bazel:actix_web",
        "//pigtail/bazel:log",
        "//pigtail/bazel:anyhow",
        "//pigtail/bazel:thiserror",
        "//pigtail/bazel:reqwest",
        "//pigtail/bazel:lazy_static",
        "//pigtail/bazel:time",
        "//pigtail/bazel:uuid",
    ],    
    proc_macro_deps = [
        "//pigtail/bazel:serde_derive",
    ]    
)

rust_binary(
    name="pigtail",
    aliases = {
        ":pigtail_lib": "pigtail"
    },
    deps=[":pigtail_lib",
        "//potboiler_common:potboiler_common",
        "//pigtail/bazel:reqwest",
        "//pigtail/bazel:actix_web",
        "//pigtail/bazel:log",
        "//pigtail/bazel:log4rs",
        "//pigtail/bazel:anyhow",    
    ],
    srcs=["src/main.rs"],
)