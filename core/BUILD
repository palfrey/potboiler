load("@io_bazel_rules_docker//rust:image.bzl", "rust_image")
load("@io_bazel_rules_rust//rust:rust.bzl", "rust_binary")

rust_binary(
    name = "potboiler",
    srcs = glob(["src/*.rs"]),
    deps = [
        "//cargo:schemamama",
        "//cargo:schemamama_postgres",
        "//cargo:postgres",
        "//cargo:log4rs",
        "//cargo:iron",
        "//cargo:router",
        "//cargo:logger",
        "//cargo:serde",
        "//cargo:serde_json",
        "//cargo:urlencoded",
        "//cargo:url",
        "//cargo:hyper",
        "//cargo:uuid",
        "//cargo:hybrid_clocks",
        "//cargo:r2d2",
        "//cargo:r2d2_postgres",
        "//cargo:persistent",
        "//potboiler_common"
    ]
)