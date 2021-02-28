load("@io_bazel_rules_docker//rust:image.bzl", "rust_image")
load("@rules_rust//rust:rust.bzl", "rust_binary", "rust_library")

rust_library(
    name = "kv_lib",
    srcs = glob(["src/*.rs"], exclude=["src/main.rs"]),
    deps = [
        "//kv/bazel:log4rs",
        "//kv/bazel:serde",
        "//kv/bazel:serde_json",
        "//kv/bazel:url",
        "//kv/bazel:hybrid_clocks",
        "//kv/bazel:r2d2",
        "//potboiler_common:potboiler_common",
        "//kv/bazel:actix_web",
        "//kv/bazel:log",
        "//kv/bazel:anyhow",
        "//kv/bazel:thiserror",
        "//kv/bazel:reqwest",
        "//kv/bazel:lazy_static",
    ],    
    proc_macro_deps = [
        "//kv/bazel:serde_derive",
    ]    
)

rust_binary(
    name="kv",
    aliases = {
        ":kv_lib": "kv"
    },
    deps=[":kv_lib",
        "//potboiler_common:potboiler_common",
        "//kv/bazel:reqwest",
        "//kv/bazel:actix_web",
        "//kv/bazel:log",
        "//kv/bazel:log4rs",
        "//kv/bazel:anyhow",    
    ],
    srcs=["src/main.rs"],
)