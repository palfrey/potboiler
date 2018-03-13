load("@io_bazel_rules_docker//rust:image.bzl", "rust_image")
load("@io_bazel_rules_rust//rust:rust.bzl", "rust_library")

rust_library(
    name = "potboiler_common",
    srcs = glob(["src/*.rs"]),
    deps = [
        "//cargo:iron",
        "//cargo:uuid",
        "//cargo:postgres",
        "//cargo:r2d2",
        "//cargo:r2d2_postgres",
        "//cargo:serde",
        "//cargo:serde_derive",
        "//cargo:serde_json",
        "//cargo:hybrid_clocks",
        "//cargo:router",
    ]
)