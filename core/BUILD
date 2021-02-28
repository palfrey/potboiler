load("@io_bazel_rules_docker//rust:image.bzl", "rust_image")
load("@rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "potboiler",
    srcs = glob(["src/*.rs"]),
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
        "//potboiler_common"
    ]
)