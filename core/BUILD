load("@io_bazel_rules_docker//rust:image.bzl", "rust_image")
load("@rules_rust//rust:rust.bzl", "rust_binary", "rust_library")

rust_library(
    name = "potboiler_lib",
    srcs = glob(["src/*.rs"], exclude=["src/main.rs"]),
    deps = [
        "//core/bazel:schemamama",
        "//core/bazel:schemamama_postgres",
        "//core/bazel:postgres",
        "//core/bazel:log4rs",
        "//core/bazel:serde",
        "//core/bazel:serde_json",
        "//core/bazel:url",
        "//core/bazel:uuid",
        "//core/bazel:hybrid_clocks",
        "//core/bazel:r2d2",
        "//core/bazel:r2d2_postgres",    
        "//potboiler_common:potboiler_common",
        "//core/bazel:actix_web",
        "//core/bazel:log",
        "//core/bazel:anyhow",
        "//core/bazel:thiserror",
        "//core/bazel:reqwest",
        "//core/bazel:trust_dns_resolver",
    ],    
    proc_macro_deps = [
        "//core/bazel:serde_derive",
    ]    
)

rust_binary(
    name="potboiler",
    aliases = {
        ":potboiler_lib": "potboiler"
    },
    deps=[":potboiler_lib",
        "//potboiler_common:potboiler_common",
        "//core/bazel:actix_web",
        "//core/bazel:log",
        "//core/bazel:log4rs",
        "//core/bazel:anyhow",    
    ],
    srcs=["src/main.rs"],
)