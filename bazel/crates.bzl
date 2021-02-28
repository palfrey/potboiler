"""
@generated
cargo-raze generated Bazel file.

DO NOT EDIT! Replaced on runs of cargo-raze
"""

load("@bazel_tools//tools/build_defs/repo:git.bzl", "new_git_repository")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")  # buildifier: disable=load
load("@bazel_tools//tools/build_defs/repo:utils.bzl", "maybe")  # buildifier: disable=load

def raze_fetch_remote_crates():
    """This function defines a collection of repos and should be called in a WORKSPACE file"""
    maybe(
        http_archive,
        name = "raze__MacTypes_sys__2_1_0",
        url = "https://crates.io/api/v1/crates/MacTypes-sys/2.1.0/download",
        type = "tar.gz",
        sha256 = "eaf9f0d0b1cc33a4d2aee14fb4b2eac03462ef4db29c8ac4057327d8a71ad86f",
        strip_prefix = "MacTypes-sys-2.1.0",
        build_file = Label("//bazel/remote:BUILD.MacTypes-sys-2.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__actix__0_7_9",
        url = "https://crates.io/api/v1/crates/actix/0.7.9/download",
        type = "tar.gz",
        sha256 = "6c616db5fa4b0c40702fb75201c2af7f8aa8f3a2e2c1dda3b0655772aa949666",
        strip_prefix = "actix-0.7.9",
        build_file = Label("//bazel/remote:BUILD.actix-0.7.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__actix_net__0_2_6",
        url = "https://crates.io/api/v1/crates/actix-net/0.2.6/download",
        type = "tar.gz",
        sha256 = "8bebfbe6629e0131730746718c9e032b58f02c6ce06ed7c982b9fef6c8545acd",
        strip_prefix = "actix-net-0.2.6",
        build_file = Label("//bazel/remote:BUILD.actix-net-0.2.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__actix_web__0_7_18",
        url = "https://crates.io/api/v1/crates/actix-web/0.7.18/download",
        type = "tar.gz",
        sha256 = "e9f33c941e5e69a58a6bfef33853228042ed3799fc4b5a4923a36a85776fb690",
        strip_prefix = "actix-web-0.7.18",
        build_file = Label("//bazel/remote:BUILD.actix-web-0.7.18.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__actix_derive__0_3_2",
        url = "https://crates.io/api/v1/crates/actix_derive/0.3.2/download",
        type = "tar.gz",
        sha256 = "4300e9431455322ae393d43a2ba1ef96b8080573c0fc23b196219efedfb6ba69",
        strip_prefix = "actix_derive-0.3.2",
        build_file = Label("//bazel/remote:BUILD.actix_derive-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__adler32__1_0_3",
        url = "https://crates.io/api/v1/crates/adler32/1.0.3/download",
        type = "tar.gz",
        sha256 = "7e522997b529f05601e05166c07ed17789691f562762c7f3b987263d2dedee5c",
        strip_prefix = "adler32-1.0.3",
        build_file = Label("//bazel/remote:BUILD.adler32-1.0.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__aho_corasick__0_6_10",
        url = "https://crates.io/api/v1/crates/aho-corasick/0.6.10/download",
        type = "tar.gz",
        sha256 = "81ce3d38065e618af2d7b77e10c5ad9a069859b4be3c2250f674af3840d9c8a5",
        strip_prefix = "aho-corasick-0.6.10",
        build_file = Label("//bazel/remote:BUILD.aho-corasick-0.6.10.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ansi_term__0_11_0",
        url = "https://crates.io/api/v1/crates/ansi_term/0.11.0/download",
        type = "tar.gz",
        sha256 = "ee49baf6cb617b853aa8d93bf420db2383fab46d314482ca2803b40d5fde979b",
        strip_prefix = "ansi_term-0.11.0",
        build_file = Label("//bazel/remote:BUILD.ansi_term-0.11.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__antidote__1_0_0",
        url = "https://crates.io/api/v1/crates/antidote/1.0.0/download",
        type = "tar.gz",
        sha256 = "34fde25430d87a9388dadbe6e34d7f72a462c8b43ac8d309b42b0a8505d7e2a5",
        strip_prefix = "antidote-1.0.0",
        build_file = Label("//bazel/remote:BUILD.antidote-1.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__anyhow__1_0_38",
        url = "https://crates.io/api/v1/crates/anyhow/1.0.38/download",
        type = "tar.gz",
        sha256 = "afddf7f520a80dbf76e6f50a35bca42a2331ef227a28b3b6dc5c2e2338d114b1",
        strip_prefix = "anyhow-1.0.38",
        build_file = Label("//bazel/remote:BUILD.anyhow-1.0.38.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__arc_swap__0_3_7",
        url = "https://crates.io/api/v1/crates/arc-swap/0.3.7/download",
        type = "tar.gz",
        sha256 = "1025aeae2b664ca0ea726a89d574fe8f4e77dd712d443236ad1de00379450cf6",
        strip_prefix = "arc-swap-0.3.7",
        build_file = Label("//bazel/remote:BUILD.arc-swap-0.3.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__arrayref__0_3_5",
        url = "https://crates.io/api/v1/crates/arrayref/0.3.5/download",
        type = "tar.gz",
        sha256 = "0d382e583f07208808f6b1249e60848879ba3543f57c32277bf52d69c2f0f0ee",
        strip_prefix = "arrayref-0.3.5",
        build_file = Label("//bazel/remote:BUILD.arrayref-0.3.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__arrayvec__0_4_10",
        url = "https://crates.io/api/v1/crates/arrayvec/0.4.10/download",
        type = "tar.gz",
        sha256 = "92c7fb76bc8826a8b33b4ee5bb07a247a81e76764ab4d55e8f73e3a4d8808c71",
        strip_prefix = "arrayvec-0.4.10",
        build_file = Label("//bazel/remote:BUILD.arrayvec-0.4.10.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__atty__0_2_11",
        url = "https://crates.io/api/v1/crates/atty/0.2.11/download",
        type = "tar.gz",
        sha256 = "9a7d5b8723950951411ee34d271d99dddcc2035a16ab25310ea2c8cfd4369652",
        strip_prefix = "atty-0.2.11",
        build_file = Label("//bazel/remote:BUILD.atty-0.2.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__autocfg__0_1_2",
        url = "https://crates.io/api/v1/crates/autocfg/0.1.2/download",
        type = "tar.gz",
        sha256 = "a6d640bee2da49f60a4068a7fae53acde8982514ab7bae8b8cea9e88cbcfd799",
        strip_prefix = "autocfg-0.1.2",
        build_file = Label("//bazel/remote:BUILD.autocfg-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__backtrace__0_3_13",
        url = "https://crates.io/api/v1/crates/backtrace/0.3.13/download",
        type = "tar.gz",
        sha256 = "b5b493b66e03090ebc4343eb02f94ff944e0cbc9ac6571491d170ba026741eb5",
        strip_prefix = "backtrace-0.3.13",
        build_file = Label("//bazel/remote:BUILD.backtrace-0.3.13.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__backtrace_sys__0_1_28",
        url = "https://crates.io/api/v1/crates/backtrace-sys/0.1.28/download",
        type = "tar.gz",
        sha256 = "797c830ac25ccc92a7f8a7b9862bde440715531514594a6154e3d4a54dd769b6",
        strip_prefix = "backtrace-sys-0.1.28",
        build_file = Label("//bazel/remote:BUILD.backtrace-sys-0.1.28.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__base64__0_10_1",
        url = "https://crates.io/api/v1/crates/base64/0.10.1/download",
        type = "tar.gz",
        sha256 = "0b25d992356d2eb0ed82172f5248873db5560c4721f564b13cb5193bda5e668e",
        strip_prefix = "base64-0.10.1",
        build_file = Label("//bazel/remote:BUILD.base64-0.10.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__base64__0_6_0",
        url = "https://crates.io/api/v1/crates/base64/0.6.0/download",
        type = "tar.gz",
        sha256 = "96434f987501f0ed4eb336a411e0631ecd1afa11574fe148587adc4ff96143c9",
        strip_prefix = "base64-0.6.0",
        build_file = Label("//bazel/remote:BUILD.base64-0.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__base64__0_9_3",
        url = "https://crates.io/api/v1/crates/base64/0.9.3/download",
        type = "tar.gz",
        sha256 = "489d6c0ed21b11d038c31b6ceccca973e65d73ba3bd8ecb9a2babf5546164643",
        strip_prefix = "base64-0.9.3",
        build_file = Label("//bazel/remote:BUILD.base64-0.9.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bitflags__1_0_4",
        url = "https://crates.io/api/v1/crates/bitflags/1.0.4/download",
        type = "tar.gz",
        sha256 = "228047a76f468627ca71776ecdebd732a3423081fcf5125585bcd7c49886ce12",
        strip_prefix = "bitflags-1.0.4",
        build_file = Label("//bazel/remote:BUILD.bitflags-1.0.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__block_buffer__0_3_3",
        url = "https://crates.io/api/v1/crates/block-buffer/0.3.3/download",
        type = "tar.gz",
        sha256 = "a076c298b9ecdb530ed9d967e74a6027d6a7478924520acddcddc24c1c8ab3ab",
        strip_prefix = "block-buffer-0.3.3",
        build_file = Label("//bazel/remote:BUILD.block-buffer-0.3.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__brotli_sys__0_3_2",
        url = "https://crates.io/api/v1/crates/brotli-sys/0.3.2/download",
        type = "tar.gz",
        sha256 = "4445dea95f4c2b41cde57cc9fee236ae4dbae88d8fcbdb4750fc1bb5d86aaecd",
        strip_prefix = "brotli-sys-0.3.2",
        build_file = Label("//bazel/remote:BUILD.brotli-sys-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__brotli2__0_3_2",
        url = "https://crates.io/api/v1/crates/brotli2/0.3.2/download",
        type = "tar.gz",
        sha256 = "0cb036c3eade309815c15ddbacec5b22c4d1f3983a774ab2eac2e3e9ea85568e",
        strip_prefix = "brotli2-0.3.2",
        build_file = Label("//bazel/remote:BUILD.brotli2-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__build_const__0_2_1",
        url = "https://crates.io/api/v1/crates/build_const/0.2.1/download",
        type = "tar.gz",
        sha256 = "39092a32794787acd8525ee150305ff051b0aa6cc2abaf193924f5ab05425f39",
        strip_prefix = "build_const-0.2.1",
        build_file = Label("//bazel/remote:BUILD.build_const-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__byte_tools__0_2_0",
        url = "https://crates.io/api/v1/crates/byte-tools/0.2.0/download",
        type = "tar.gz",
        sha256 = "560c32574a12a89ecd91f5e742165893f86e3ab98d21f8ea548658eb9eef5f40",
        strip_prefix = "byte-tools-0.2.0",
        build_file = Label("//bazel/remote:BUILD.byte-tools-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__byteorder__0_5_3",
        url = "https://crates.io/api/v1/crates/byteorder/0.5.3/download",
        type = "tar.gz",
        sha256 = "0fc10e8cc6b2580fda3f36eb6dc5316657f812a3df879a44a66fc9f0fdbc4855",
        strip_prefix = "byteorder-0.5.3",
        build_file = Label("//bazel/remote:BUILD.byteorder-0.5.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__byteorder__1_3_1",
        url = "https://crates.io/api/v1/crates/byteorder/1.3.1/download",
        type = "tar.gz",
        sha256 = "a019b10a2a7cdeb292db131fc8113e57ea2a908f6e7894b0c3c671893b65dbeb",
        strip_prefix = "byteorder-1.3.1",
        build_file = Label("//bazel/remote:BUILD.byteorder-1.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__bytes__0_4_11",
        url = "https://crates.io/api/v1/crates/bytes/0.4.11/download",
        type = "tar.gz",
        sha256 = "40ade3d27603c2cb345eb0912aec461a6dec7e06a4ae48589904e808335c7afa",
        strip_prefix = "bytes-0.4.11",
        build_file = Label("//bazel/remote:BUILD.bytes-0.4.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cc__1_0_29",
        url = "https://crates.io/api/v1/crates/cc/1.0.29/download",
        type = "tar.gz",
        sha256 = "4390a3b5f4f6bce9c1d0c00128379df433e53777fdd30e92f16a529332baec4e",
        strip_prefix = "cc-1.0.29",
        build_file = Label("//bazel/remote:BUILD.cc-1.0.29.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cfg_if__0_1_6",
        url = "https://crates.io/api/v1/crates/cfg-if/0.1.6/download",
        type = "tar.gz",
        sha256 = "082bb9b28e00d3c9d39cc03e64ce4cea0f1bb9b3fde493f0cbc008472d22bdf4",
        strip_prefix = "cfg-if-0.1.6",
        build_file = Label("//bazel/remote:BUILD.cfg-if-0.1.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__chrono__0_4_6",
        url = "https://crates.io/api/v1/crates/chrono/0.4.6/download",
        type = "tar.gz",
        sha256 = "45912881121cb26fad7c38c17ba7daa18764771836b34fab7d3fbd93ed633878",
        strip_prefix = "chrono-0.4.6",
        build_file = Label("//bazel/remote:BUILD.chrono-0.4.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cloudabi__0_0_3",
        url = "https://crates.io/api/v1/crates/cloudabi/0.0.3/download",
        type = "tar.gz",
        sha256 = "ddfc5b9aa5d4507acaf872de71051dfd0e309860e88966e1051e462a077aac4f",
        strip_prefix = "cloudabi-0.0.3",
        build_file = Label("//bazel/remote:BUILD.cloudabi-0.0.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__colored__1_7_0",
        url = "https://crates.io/api/v1/crates/colored/1.7.0/download",
        type = "tar.gz",
        sha256 = "6e9a455e156a4271e12fd0246238c380b1e223e3736663c7a18ed8b6362028a9",
        strip_prefix = "colored-1.7.0",
        build_file = Label("//bazel/remote:BUILD.colored-1.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__constant_time_eq__0_1_3",
        url = "https://crates.io/api/v1/crates/constant_time_eq/0.1.3/download",
        type = "tar.gz",
        sha256 = "8ff012e225ce166d4422e0e78419d901719760f62ae2b7969ca6b564d1b54a9e",
        strip_prefix = "constant_time_eq-0.1.3",
        build_file = Label("//bazel/remote:BUILD.constant_time_eq-0.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__cookie__0_11_0",
        url = "https://crates.io/api/v1/crates/cookie/0.11.0/download",
        type = "tar.gz",
        sha256 = "1465f8134efa296b4c19db34d909637cb2bf0f7aaf21299e23e18fa29ac557cf",
        strip_prefix = "cookie-0.11.0",
        build_file = Label("//bazel/remote:BUILD.cookie-0.11.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__core_foundation__0_5_1",
        url = "https://crates.io/api/v1/crates/core-foundation/0.5.1/download",
        type = "tar.gz",
        sha256 = "286e0b41c3a20da26536c6000a280585d519fd07b3956b43aed8a79e9edce980",
        strip_prefix = "core-foundation-0.5.1",
        build_file = Label("//bazel/remote:BUILD.core-foundation-0.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__core_foundation_sys__0_5_1",
        url = "https://crates.io/api/v1/crates/core-foundation-sys/0.5.1/download",
        type = "tar.gz",
        sha256 = "716c271e8613ace48344f723b60b900a93150271e5be206212d052bbc0883efa",
        strip_prefix = "core-foundation-sys-0.5.1",
        build_file = Label("//bazel/remote:BUILD.core-foundation-sys-0.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crc__1_8_1",
        url = "https://crates.io/api/v1/crates/crc/1.8.1/download",
        type = "tar.gz",
        sha256 = "d663548de7f5cca343f1e0a48d14dcfb0e9eb4e079ec58883b7251539fa10aeb",
        strip_prefix = "crc-1.8.1",
        build_file = Label("//bazel/remote:BUILD.crc-1.8.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crc32fast__1_1_2",
        url = "https://crates.io/api/v1/crates/crc32fast/1.1.2/download",
        type = "tar.gz",
        sha256 = "e91d5240c6975ef33aeb5f148f35275c25eda8e8a5f95abe421978b05b8bf192",
        strip_prefix = "crc32fast-1.1.2",
        build_file = Label("//bazel/remote:BUILD.crc32fast-1.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crossbeam__0_3_2",
        url = "https://crates.io/api/v1/crates/crossbeam/0.3.2/download",
        type = "tar.gz",
        sha256 = "24ce9782d4d5c53674646a6a4c1863a21a8fc0cb649b3c94dfc16e45071dea19",
        strip_prefix = "crossbeam-0.3.2",
        build_file = Label("//bazel/remote:BUILD.crossbeam-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crossbeam__0_6_0",
        url = "https://crates.io/api/v1/crates/crossbeam/0.6.0/download",
        type = "tar.gz",
        sha256 = "ad4c7ea749d9fb09e23c5cb17e3b70650860553a0e2744e38446b1803bf7db94",
        strip_prefix = "crossbeam-0.6.0",
        build_file = Label("//bazel/remote:BUILD.crossbeam-0.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crossbeam_channel__0_3_8",
        url = "https://crates.io/api/v1/crates/crossbeam-channel/0.3.8/download",
        type = "tar.gz",
        sha256 = "0f0ed1a4de2235cabda8558ff5840bffb97fcb64c97827f354a451307df5f72b",
        strip_prefix = "crossbeam-channel-0.3.8",
        build_file = Label("//bazel/remote:BUILD.crossbeam-channel-0.3.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crossbeam_deque__0_6_3",
        url = "https://crates.io/api/v1/crates/crossbeam-deque/0.6.3/download",
        type = "tar.gz",
        sha256 = "05e44b8cf3e1a625844d1750e1f7820da46044ff6d28f4d43e455ba3e5bb2c13",
        strip_prefix = "crossbeam-deque-0.6.3",
        build_file = Label("//bazel/remote:BUILD.crossbeam-deque-0.6.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crossbeam_epoch__0_7_1",
        url = "https://crates.io/api/v1/crates/crossbeam-epoch/0.7.1/download",
        type = "tar.gz",
        sha256 = "04c9e3102cc2d69cd681412141b390abd55a362afc1540965dad0ad4d34280b4",
        strip_prefix = "crossbeam-epoch-0.7.1",
        build_file = Label("//bazel/remote:BUILD.crossbeam-epoch-0.7.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crossbeam_utils__0_6_5",
        url = "https://crates.io/api/v1/crates/crossbeam-utils/0.6.5/download",
        type = "tar.gz",
        sha256 = "f8306fcef4a7b563b76b7dd949ca48f52bc1141aa067d2ea09565f3e2652aa5c",
        strip_prefix = "crossbeam-utils-0.6.5",
        build_file = Label("//bazel/remote:BUILD.crossbeam-utils-0.6.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__crypto_mac__0_5_2",
        url = "https://crates.io/api/v1/crates/crypto-mac/0.5.2/download",
        type = "tar.gz",
        sha256 = "0999b4ff4d3446d4ddb19a63e9e00c1876e75cd7000d20e57a693b4b3f08d958",
        strip_prefix = "crypto-mac-0.5.2",
        build_file = Label("//bazel/remote:BUILD.crypto-mac-0.5.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__difference__2_0_0",
        url = "https://crates.io/api/v1/crates/difference/2.0.0/download",
        type = "tar.gz",
        sha256 = "524cbf6897b527295dff137cec09ecf3a05f4fddffd7dfcd1585403449e74198",
        strip_prefix = "difference-2.0.0",
        build_file = Label("//bazel/remote:BUILD.difference-2.0.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__digest__0_7_6",
        url = "https://crates.io/api/v1/crates/digest/0.7.6/download",
        type = "tar.gz",
        sha256 = "03b072242a8cbaf9c145665af9d250c59af3b958f83ed6824e13533cf76d5b90",
        strip_prefix = "digest-0.7.6",
        build_file = Label("//bazel/remote:BUILD.digest-0.7.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__dtoa__0_4_3",
        url = "https://crates.io/api/v1/crates/dtoa/0.4.3/download",
        type = "tar.gz",
        sha256 = "6d301140eb411af13d3115f9a562c85cc6b541ade9dfa314132244aaee7489dd",
        strip_prefix = "dtoa-0.4.3",
        build_file = Label("//bazel/remote:BUILD.dtoa-0.4.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding__0_2_33",
        url = "https://crates.io/api/v1/crates/encoding/0.2.33/download",
        type = "tar.gz",
        sha256 = "6b0d943856b990d12d3b55b359144ff341533e516d94098b1d3fc1ac666d36ec",
        strip_prefix = "encoding-0.2.33",
        build_file = Label("//bazel/remote:BUILD.encoding-0.2.33.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding_index_japanese__1_20141219_5",
        url = "https://crates.io/api/v1/crates/encoding-index-japanese/1.20141219.5/download",
        type = "tar.gz",
        sha256 = "04e8b2ff42e9a05335dbf8b5c6f7567e5591d0d916ccef4e0b1710d32a0d0c91",
        strip_prefix = "encoding-index-japanese-1.20141219.5",
        build_file = Label("//bazel/remote:BUILD.encoding-index-japanese-1.20141219.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding_index_korean__1_20141219_5",
        url = "https://crates.io/api/v1/crates/encoding-index-korean/1.20141219.5/download",
        type = "tar.gz",
        sha256 = "4dc33fb8e6bcba213fe2f14275f0963fd16f0a02c878e3095ecfdf5bee529d81",
        strip_prefix = "encoding-index-korean-1.20141219.5",
        build_file = Label("//bazel/remote:BUILD.encoding-index-korean-1.20141219.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding_index_simpchinese__1_20141219_5",
        url = "https://crates.io/api/v1/crates/encoding-index-simpchinese/1.20141219.5/download",
        type = "tar.gz",
        sha256 = "d87a7194909b9118fc707194baa434a4e3b0fb6a5a757c73c3adb07aa25031f7",
        strip_prefix = "encoding-index-simpchinese-1.20141219.5",
        build_file = Label("//bazel/remote:BUILD.encoding-index-simpchinese-1.20141219.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding_index_singlebyte__1_20141219_5",
        url = "https://crates.io/api/v1/crates/encoding-index-singlebyte/1.20141219.5/download",
        type = "tar.gz",
        sha256 = "3351d5acffb224af9ca265f435b859c7c01537c0849754d3db3fdf2bfe2ae84a",
        strip_prefix = "encoding-index-singlebyte-1.20141219.5",
        build_file = Label("//bazel/remote:BUILD.encoding-index-singlebyte-1.20141219.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding_index_tradchinese__1_20141219_5",
        url = "https://crates.io/api/v1/crates/encoding-index-tradchinese/1.20141219.5/download",
        type = "tar.gz",
        sha256 = "fd0e20d5688ce3cab59eb3ef3a2083a5c77bf496cb798dc6fcdb75f323890c18",
        strip_prefix = "encoding-index-tradchinese-1.20141219.5",
        build_file = Label("//bazel/remote:BUILD.encoding-index-tradchinese-1.20141219.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding_index_tests__0_1_4",
        url = "https://crates.io/api/v1/crates/encoding_index_tests/0.1.4/download",
        type = "tar.gz",
        sha256 = "a246d82be1c9d791c5dfde9a2bd045fc3cbba3fa2b11ad558f27d01712f00569",
        strip_prefix = "encoding_index_tests-0.1.4",
        build_file = Label("//bazel/remote:BUILD.encoding_index_tests-0.1.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__encoding_rs__0_8_16",
        url = "https://crates.io/api/v1/crates/encoding_rs/0.8.16/download",
        type = "tar.gz",
        sha256 = "0535f350c60aac0b87ccf28319abc749391e912192255b0c00a2c12c6917bd73",
        strip_prefix = "encoding_rs-0.8.16",
        build_file = Label("//bazel/remote:BUILD.encoding_rs-0.8.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__env_logger__0_6_0",
        url = "https://crates.io/api/v1/crates/env_logger/0.6.0/download",
        type = "tar.gz",
        sha256 = "afb070faf94c85d17d50ca44f6ad076bce18ae92f0037d350947240a36e9d42e",
        strip_prefix = "env_logger-0.6.0",
        build_file = Label("//bazel/remote:BUILD.env_logger-0.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__error_chain__0_8_1",
        url = "https://crates.io/api/v1/crates/error-chain/0.8.1/download",
        type = "tar.gz",
        sha256 = "6930e04918388a9a2e41d518c25cf679ccafe26733fb4127dbf21993f2575d46",
        strip_prefix = "error-chain-0.8.1",
        build_file = Label("//bazel/remote:BUILD.error-chain-0.8.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__failure__0_1_5",
        url = "https://crates.io/api/v1/crates/failure/0.1.5/download",
        type = "tar.gz",
        sha256 = "795bd83d3abeb9220f257e597aa0080a508b27533824adf336529648f6abf7e2",
        strip_prefix = "failure-0.1.5",
        build_file = Label("//bazel/remote:BUILD.failure-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__failure_derive__0_1_5",
        url = "https://crates.io/api/v1/crates/failure_derive/0.1.5/download",
        type = "tar.gz",
        sha256 = "ea1063915fd7ef4309e222a5a07cf9c319fb9c7836b1f89b85458672dbb127e1",
        strip_prefix = "failure_derive-0.1.5",
        build_file = Label("//bazel/remote:BUILD.failure_derive-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fake_simd__0_1_2",
        url = "https://crates.io/api/v1/crates/fake-simd/0.1.2/download",
        type = "tar.gz",
        sha256 = "e88a8acf291dafb59c2d96e8f59828f3838bb1a70398823ade51a84de6a6deed",
        strip_prefix = "fake-simd-0.1.2",
        build_file = Label("//bazel/remote:BUILD.fake-simd-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fallible_iterator__0_1_6",
        url = "https://crates.io/api/v1/crates/fallible-iterator/0.1.6/download",
        type = "tar.gz",
        sha256 = "eb7217124812dc5672b7476d0c2d20cfe9f7c0f1ba0904b674a9762a0212f72e",
        strip_prefix = "fallible-iterator-0.1.6",
        build_file = Label("//bazel/remote:BUILD.fallible-iterator-0.1.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__flate2__1_0_6",
        url = "https://crates.io/api/v1/crates/flate2/1.0.6/download",
        type = "tar.gz",
        sha256 = "2291c165c8e703ee54ef3055ad6188e3d51108e2ded18e9f2476e774fc5ad3d4",
        strip_prefix = "flate2-1.0.6",
        build_file = Label("//bazel/remote:BUILD.flate2-1.0.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fnv__1_0_6",
        url = "https://crates.io/api/v1/crates/fnv/1.0.6/download",
        type = "tar.gz",
        sha256 = "2fad85553e09a6f881f739c29f0b00b0f01357c743266d478b68951ce23285f3",
        strip_prefix = "fnv-1.0.6",
        build_file = Label("//bazel/remote:BUILD.fnv-1.0.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__foreign_types__0_3_2",
        url = "https://crates.io/api/v1/crates/foreign-types/0.3.2/download",
        type = "tar.gz",
        sha256 = "f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1",
        strip_prefix = "foreign-types-0.3.2",
        build_file = Label("//bazel/remote:BUILD.foreign-types-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__foreign_types_shared__0_1_1",
        url = "https://crates.io/api/v1/crates/foreign-types-shared/0.1.1/download",
        type = "tar.gz",
        sha256 = "00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b",
        strip_prefix = "foreign-types-shared-0.1.1",
        build_file = Label("//bazel/remote:BUILD.foreign-types-shared-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fuchsia_cprng__0_1_1",
        url = "https://crates.io/api/v1/crates/fuchsia-cprng/0.1.1/download",
        type = "tar.gz",
        sha256 = "a06f77d526c1a601b7c4cdd98f54b5eaabffc14d5f2f0296febdc7f357c6d3ba",
        strip_prefix = "fuchsia-cprng-0.1.1",
        build_file = Label("//bazel/remote:BUILD.fuchsia-cprng-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fuchsia_zircon__0_3_3",
        url = "https://crates.io/api/v1/crates/fuchsia-zircon/0.3.3/download",
        type = "tar.gz",
        sha256 = "2e9763c69ebaae630ba35f74888db465e49e259ba1bc0eda7d06f4a067615d82",
        strip_prefix = "fuchsia-zircon-0.3.3",
        build_file = Label("//bazel/remote:BUILD.fuchsia-zircon-0.3.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__fuchsia_zircon_sys__0_3_3",
        url = "https://crates.io/api/v1/crates/fuchsia-zircon-sys/0.3.3/download",
        type = "tar.gz",
        sha256 = "3dcaa9ae7725d12cdb85b3ad99a434db70b468c09ded17e012d86b5c1010f7a7",
        strip_prefix = "fuchsia-zircon-sys-0.3.3",
        build_file = Label("//bazel/remote:BUILD.fuchsia-zircon-sys-0.3.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures__0_1_25",
        url = "https://crates.io/api/v1/crates/futures/0.1.25/download",
        type = "tar.gz",
        sha256 = "49e7653e374fe0d0c12de4250f0bdb60680b8c80eed558c5c7538eec9c89e21b",
        strip_prefix = "futures-0.1.25",
        build_file = Label("//bazel/remote:BUILD.futures-0.1.25.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__futures_cpupool__0_1_8",
        url = "https://crates.io/api/v1/crates/futures-cpupool/0.1.8/download",
        type = "tar.gz",
        sha256 = "ab90cde24b3319636588d0c35fe03b1333857621051837ed769faefb4c2162e4",
        strip_prefix = "futures-cpupool-0.1.8",
        build_file = Label("//bazel/remote:BUILD.futures-cpupool-0.1.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__generic_array__0_9_0",
        url = "https://crates.io/api/v1/crates/generic-array/0.9.0/download",
        type = "tar.gz",
        sha256 = "ef25c5683767570c2bbd7deba372926a55eaae9982d7726ee2a1050239d45b9d",
        strip_prefix = "generic-array-0.9.0",
        build_file = Label("//bazel/remote:BUILD.generic-array-0.9.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__h2__0_1_16",
        url = "https://crates.io/api/v1/crates/h2/0.1.16/download",
        type = "tar.gz",
        sha256 = "ddb2b25a33e231484694267af28fec74ac63b5ccf51ee2065a5e313b834d836e",
        strip_prefix = "h2-0.1.16",
        build_file = Label("//bazel/remote:BUILD.h2-0.1.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hex__0_2_0",
        url = "https://crates.io/api/v1/crates/hex/0.2.0/download",
        type = "tar.gz",
        sha256 = "d6a22814455d41612f41161581c2883c0c6a1c41852729b17d5ed88f01e153aa",
        strip_prefix = "hex-0.2.0",
        build_file = Label("//bazel/remote:BUILD.hex-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hex_slice__0_1_4",
        url = "https://crates.io/api/v1/crates/hex-slice/0.1.4/download",
        type = "tar.gz",
        sha256 = "5491a308e0214554f07a81d8944abe45f552871c12e3c3c6e7e5d354039a6c4c",
        strip_prefix = "hex-slice-0.1.4",
        build_file = Label("//bazel/remote:BUILD.hex-slice-0.1.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hmac__0_5_0",
        url = "https://crates.io/api/v1/crates/hmac/0.5.0/download",
        type = "tar.gz",
        sha256 = "44f3bdb08579d99d7dc761c0e266f13b5f2ab8c8c703b9fc9ef333cd8f48f55e",
        strip_prefix = "hmac-0.5.0",
        build_file = Label("//bazel/remote:BUILD.hmac-0.5.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hostname__0_1_5",
        url = "https://crates.io/api/v1/crates/hostname/0.1.5/download",
        type = "tar.gz",
        sha256 = "21ceb46a83a85e824ef93669c8b390009623863b5c195d1ba747292c0c72f94e",
        strip_prefix = "hostname-0.1.5",
        build_file = Label("//bazel/remote:BUILD.hostname-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__http__0_1_21",
        url = "https://crates.io/api/v1/crates/http/0.1.21/download",
        type = "tar.gz",
        sha256 = "d6ccf5ede3a895d8856620237b2f02972c1bbc78d2965ad7fe8838d4a0ed41f0",
        strip_prefix = "http-0.1.21",
        build_file = Label("//bazel/remote:BUILD.http-0.1.21.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__httparse__1_3_3",
        url = "https://crates.io/api/v1/crates/httparse/1.3.3/download",
        type = "tar.gz",
        sha256 = "e8734b0cfd3bc3e101ec59100e101c2eecd19282202e87808b3037b442777a83",
        strip_prefix = "httparse-1.3.3",
        build_file = Label("//bazel/remote:BUILD.httparse-1.3.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__humantime__1_2_0",
        url = "https://crates.io/api/v1/crates/humantime/1.2.0/download",
        type = "tar.gz",
        sha256 = "3ca7e5f2e110db35f93b837c81797f3714500b81d517bf20c431b16d3ca4f114",
        strip_prefix = "humantime-1.2.0",
        build_file = Label("//bazel/remote:BUILD.humantime-1.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hybrid_clocks__0_3_4",
        url = "https://crates.io/api/v1/crates/hybrid-clocks/0.3.4/download",
        type = "tar.gz",
        sha256 = "c0ab83488abdea201c2ec2ca259a26638c174e2f4a511146180e2d65fe421c25",
        strip_prefix = "hybrid-clocks-0.3.4",
        build_file = Label("//bazel/remote:BUILD.hybrid-clocks-0.3.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hyper__0_12_24",
        url = "https://crates.io/api/v1/crates/hyper/0.12.24/download",
        type = "tar.gz",
        sha256 = "fdfa9b401ef6c4229745bb6e9b2529192d07b920eed624cdee2a82348cd550af",
        strip_prefix = "hyper-0.12.24",
        build_file = Label("//bazel/remote:BUILD.hyper-0.12.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__hyper_tls__0_3_1",
        url = "https://crates.io/api/v1/crates/hyper-tls/0.3.1/download",
        type = "tar.gz",
        sha256 = "32cd73f14ad370d3b4d4b7dce08f69b81536c82e39fcc89731930fe5788cd661",
        strip_prefix = "hyper-tls-0.3.1",
        build_file = Label("//bazel/remote:BUILD.hyper-tls-0.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__idna__0_1_5",
        url = "https://crates.io/api/v1/crates/idna/0.1.5/download",
        type = "tar.gz",
        sha256 = "38f09e0f0b1fb55fdee1f17470ad800da77af5186a1a76c026b679358b7e844e",
        strip_prefix = "idna-0.1.5",
        build_file = Label("//bazel/remote:BUILD.idna-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__indexmap__1_0_2",
        url = "https://crates.io/api/v1/crates/indexmap/1.0.2/download",
        type = "tar.gz",
        sha256 = "7e81a7c05f79578dbc15793d8b619db9ba32b4577003ef3af1a91c416798c58d",
        strip_prefix = "indexmap-1.0.2",
        build_file = Label("//bazel/remote:BUILD.indexmap-1.0.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__iovec__0_1_2",
        url = "https://crates.io/api/v1/crates/iovec/0.1.2/download",
        type = "tar.gz",
        sha256 = "dbe6e417e7d0975db6512b90796e8ce223145ac4e33c377e4a42882a0e88bb08",
        strip_prefix = "iovec-0.1.2",
        build_file = Label("//bazel/remote:BUILD.iovec-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ipconfig__0_1_9",
        url = "https://crates.io/api/v1/crates/ipconfig/0.1.9/download",
        type = "tar.gz",
        sha256 = "08f7eadeaf4b52700de180d147c4805f199854600b36faa963d91114827b2ffc",
        strip_prefix = "ipconfig-0.1.9",
        build_file = Label("//bazel/remote:BUILD.ipconfig-0.1.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__itoa__0_4_3",
        url = "https://crates.io/api/v1/crates/itoa/0.4.3/download",
        type = "tar.gz",
        sha256 = "1306f3464951f30e30d12373d31c79fbd52d236e5e896fd92f96ec7babbbe60b",
        strip_prefix = "itoa-0.4.3",
        build_file = Label("//bazel/remote:BUILD.itoa-0.4.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__kernel32_sys__0_2_2",
        url = "https://crates.io/api/v1/crates/kernel32-sys/0.2.2/download",
        type = "tar.gz",
        sha256 = "7507624b29483431c0ba2d82aece8ca6cdba9382bff4ddd0f7490560c056098d",
        strip_prefix = "kernel32-sys-0.2.2",
        build_file = Label("//bazel/remote:BUILD.kernel32-sys-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__language_tags__0_2_2",
        url = "https://crates.io/api/v1/crates/language-tags/0.2.2/download",
        type = "tar.gz",
        sha256 = "a91d884b6667cd606bb5a69aa0c99ba811a115fc68915e7056ec08a46e93199a",
        strip_prefix = "language-tags-0.2.2",
        build_file = Label("//bazel/remote:BUILD.language-tags-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lazy_static__1_2_0",
        url = "https://crates.io/api/v1/crates/lazy_static/1.2.0/download",
        type = "tar.gz",
        sha256 = "a374c89b9db55895453a74c1e38861d9deec0b01b405a82516e9d5de4820dea1",
        strip_prefix = "lazy_static-1.2.0",
        build_file = Label("//bazel/remote:BUILD.lazy_static-1.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lazycell__1_2_1",
        url = "https://crates.io/api/v1/crates/lazycell/1.2.1/download",
        type = "tar.gz",
        sha256 = "b294d6fa9ee409a054354afc4352b0b9ef7ca222c69b8812cbea9e7d2bf3783f",
        strip_prefix = "lazycell-1.2.1",
        build_file = Label("//bazel/remote:BUILD.lazycell-1.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__libc__0_2_48",
        url = "https://crates.io/api/v1/crates/libc/0.2.48/download",
        type = "tar.gz",
        sha256 = "e962c7641008ac010fa60a7dfdc1712449f29c44ef2d4702394aea943ee75047",
        strip_prefix = "libc-0.2.48",
        build_file = Label("//bazel/remote:BUILD.libc-0.2.48.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__libflate__0_1_25",
        url = "https://crates.io/api/v1/crates/libflate/0.1.25/download",
        type = "tar.gz",
        sha256 = "90c6f86f4b0caa347206f916f8b687b51d77c6ef8ff18d52dd007491fd580529",
        strip_prefix = "libflate-0.1.25",
        build_file = Label("//bazel/remote:BUILD.libflate-0.1.25.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__linked_hash_map__0_4_2",
        url = "https://crates.io/api/v1/crates/linked-hash-map/0.4.2/download",
        type = "tar.gz",
        sha256 = "7860ec297f7008ff7a1e3382d7f7e1dcd69efc94751a2284bafc3d013c2aa939",
        strip_prefix = "linked-hash-map-0.4.2",
        build_file = Label("//bazel/remote:BUILD.linked-hash-map-0.4.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__linked_hash_map__0_5_1",
        url = "https://crates.io/api/v1/crates/linked-hash-map/0.5.1/download",
        type = "tar.gz",
        sha256 = "70fb39025bc7cdd76305867c4eccf2f2dcf6e9a57f5b21a93e1c2d86cd03ec9e",
        strip_prefix = "linked-hash-map-0.5.1",
        build_file = Label("//bazel/remote:BUILD.linked-hash-map-0.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lock_api__0_1_5",
        url = "https://crates.io/api/v1/crates/lock_api/0.1.5/download",
        type = "tar.gz",
        sha256 = "62ebf1391f6acad60e5c8b43706dde4582df75c06698ab44511d15016bc2442c",
        strip_prefix = "lock_api-0.1.5",
        build_file = Label("//bazel/remote:BUILD.lock_api-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__log__0_3_9",
        url = "https://crates.io/api/v1/crates/log/0.3.9/download",
        type = "tar.gz",
        sha256 = "e19e8d5c34a3e0e2223db8e060f9e8264aeeb5c5fc64a4ee9965c062211c024b",
        strip_prefix = "log-0.3.9",
        build_file = Label("//bazel/remote:BUILD.log-0.3.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__log__0_4_6",
        url = "https://crates.io/api/v1/crates/log/0.4.6/download",
        type = "tar.gz",
        sha256 = "c84ec4b527950aa83a329754b01dbe3f58361d1c5efacd1f6d68c494d08a17c6",
        strip_prefix = "log-0.4.6",
        build_file = Label("//bazel/remote:BUILD.log-0.4.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__log_mdc__0_1_0",
        url = "https://crates.io/api/v1/crates/log-mdc/0.1.0/download",
        type = "tar.gz",
        sha256 = "a94d21414c1f4a51209ad204c1776a3d0765002c76c6abcb602a6f09f1e881c7",
        strip_prefix = "log-mdc-0.1.0",
        build_file = Label("//bazel/remote:BUILD.log-mdc-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__log4rs__0_8_1",
        url = "https://crates.io/api/v1/crates/log4rs/0.8.1/download",
        type = "tar.gz",
        sha256 = "25e0fc8737a634116a2deb38d821e4400ed16ce9dcb0d628a978d399260f5902",
        strip_prefix = "log4rs-0.8.1",
        build_file = Label("//bazel/remote:BUILD.log4rs-0.8.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__lru_cache__0_1_1",
        url = "https://crates.io/api/v1/crates/lru-cache/0.1.1/download",
        type = "tar.gz",
        sha256 = "4d06ff7ff06f729ce5f4e227876cb88d10bc59cd4ae1e09fbb2bde15c850dc21",
        strip_prefix = "lru-cache-0.1.1",
        build_file = Label("//bazel/remote:BUILD.lru-cache-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__matches__0_1_8",
        url = "https://crates.io/api/v1/crates/matches/0.1.8/download",
        type = "tar.gz",
        sha256 = "7ffc5c5338469d4d3ea17d269fa8ea3512ad247247c30bd2df69e68309ed0a08",
        strip_prefix = "matches-0.1.8",
        build_file = Label("//bazel/remote:BUILD.matches-0.1.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__md5__0_3_8",
        url = "https://crates.io/api/v1/crates/md5/0.3.8/download",
        type = "tar.gz",
        sha256 = "79c56d6a0b07f9e19282511c83fc5b086364cbae4ba8c7d5f190c3d9b0425a48",
        strip_prefix = "md5-0.3.8",
        build_file = Label("//bazel/remote:BUILD.md5-0.3.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__memchr__1_0_2",
        url = "https://crates.io/api/v1/crates/memchr/1.0.2/download",
        type = "tar.gz",
        sha256 = "148fab2e51b4f1cfc66da2a7c32981d1d3c083a803978268bb11fe4b86925e7a",
        strip_prefix = "memchr-1.0.2",
        build_file = Label("//bazel/remote:BUILD.memchr-1.0.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__memchr__2_2_0",
        url = "https://crates.io/api/v1/crates/memchr/2.2.0/download",
        type = "tar.gz",
        sha256 = "2efc7bc57c883d4a4d6e3246905283d8dae951bb3bd32f49d6ef297f546e1c39",
        strip_prefix = "memchr-2.2.0",
        build_file = Label("//bazel/remote:BUILD.memchr-2.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__memoffset__0_2_1",
        url = "https://crates.io/api/v1/crates/memoffset/0.2.1/download",
        type = "tar.gz",
        sha256 = "0f9dc261e2b62d7a622bf416ea3c5245cdd5d9a7fcc428c0d06804dfce1775b3",
        strip_prefix = "memoffset-0.2.1",
        build_file = Label("//bazel/remote:BUILD.memoffset-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mime__0_3_13",
        url = "https://crates.io/api/v1/crates/mime/0.3.13/download",
        type = "tar.gz",
        sha256 = "3e27ca21f40a310bd06d9031785f4801710d566c184a6e15bad4f1d9b65f9425",
        strip_prefix = "mime-0.3.13",
        build_file = Label("//bazel/remote:BUILD.mime-0.3.13.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mime_guess__2_0_0_alpha_6",
        url = "https://crates.io/api/v1/crates/mime_guess/2.0.0-alpha.6/download",
        type = "tar.gz",
        sha256 = "30de2e4613efcba1ec63d8133f344076952090c122992a903359be5a4f99c3ed",
        strip_prefix = "mime_guess-2.0.0-alpha.6",
        build_file = Label("//bazel/remote:BUILD.mime_guess-2.0.0-alpha.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__miniz_sys__0_1_11",
        url = "https://crates.io/api/v1/crates/miniz-sys/0.1.11/download",
        type = "tar.gz",
        sha256 = "0300eafb20369952951699b68243ab4334f4b10a88f411c221d444b36c40e649",
        strip_prefix = "miniz-sys-0.1.11",
        build_file = Label("//bazel/remote:BUILD.miniz-sys-0.1.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__miniz_oxide__0_2_1",
        url = "https://crates.io/api/v1/crates/miniz_oxide/0.2.1/download",
        type = "tar.gz",
        sha256 = "c468f2369f07d651a5d0bb2c9079f8488a66d5466efe42d0c5c6466edcb7f71e",
        strip_prefix = "miniz_oxide-0.2.1",
        build_file = Label("//bazel/remote:BUILD.miniz_oxide-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__miniz_oxide_c_api__0_2_1",
        url = "https://crates.io/api/v1/crates/miniz_oxide_c_api/0.2.1/download",
        type = "tar.gz",
        sha256 = "b7fe927a42e3807ef71defb191dc87d4e24479b221e67015fe38ae2b7b447bab",
        strip_prefix = "miniz_oxide_c_api-0.2.1",
        build_file = Label("//bazel/remote:BUILD.miniz_oxide_c_api-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mio__0_6_16",
        url = "https://crates.io/api/v1/crates/mio/0.6.16/download",
        type = "tar.gz",
        sha256 = "71646331f2619b1026cc302f87a2b8b648d5c6dd6937846a16cc8ce0f347f432",
        strip_prefix = "mio-0.6.16",
        build_file = Label("//bazel/remote:BUILD.mio-0.6.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mio_uds__0_6_7",
        url = "https://crates.io/api/v1/crates/mio-uds/0.6.7/download",
        type = "tar.gz",
        sha256 = "966257a94e196b11bb43aca423754d87429960a768de9414f3691d6957abf125",
        strip_prefix = "mio-uds-0.6.7",
        build_file = Label("//bazel/remote:BUILD.mio-uds-0.6.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__miow__0_2_1",
        url = "https://crates.io/api/v1/crates/miow/0.2.1/download",
        type = "tar.gz",
        sha256 = "8c1f2f3b1cf331de6896aabf6e9d55dca90356cc9960cca7eaaf408a355ae919",
        strip_prefix = "miow-0.2.1",
        build_file = Label("//bazel/remote:BUILD.miow-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__mockito__0_14_1",
        url = "https://crates.io/api/v1/crates/mockito/0.14.1/download",
        type = "tar.gz",
        sha256 = "466ec7bc68f7188b587bdf1b0857eca98de58ce63efa6adcd0e98be3ba297570",
        strip_prefix = "mockito-0.14.1",
        build_file = Label("//bazel/remote:BUILD.mockito-0.14.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__native_tls__0_2_2",
        url = "https://crates.io/api/v1/crates/native-tls/0.2.2/download",
        type = "tar.gz",
        sha256 = "ff8e08de0070bbf4c31f452ea2a70db092f36f6f2e4d897adf5674477d488fb2",
        strip_prefix = "native-tls-0.2.2",
        build_file = Label("//bazel/remote:BUILD.native-tls-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__net2__0_2_33",
        url = "https://crates.io/api/v1/crates/net2/0.2.33/download",
        type = "tar.gz",
        sha256 = "42550d9fb7b6684a6d404d9fa7250c2eb2646df731d1c06afc06dcee9e1bcf88",
        strip_prefix = "net2-0.2.33",
        build_file = Label("//bazel/remote:BUILD.net2-0.2.33.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__nodrop__0_1_13",
        url = "https://crates.io/api/v1/crates/nodrop/0.1.13/download",
        type = "tar.gz",
        sha256 = "2f9667ddcc6cc8a43afc9b7917599d7216aa09c463919ea32c59ed6cac8bc945",
        strip_prefix = "nodrop-0.1.13",
        build_file = Label("//bazel/remote:BUILD.nodrop-0.1.13.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__nom__4_2_0",
        url = "https://crates.io/api/v1/crates/nom/4.2.0/download",
        type = "tar.gz",
        sha256 = "b30adc557058ce00c9d0d7cb3c6e0b5bc6f36e2e2eabe74b0ba726d194abd588",
        strip_prefix = "nom-4.2.0",
        build_file = Label("//bazel/remote:BUILD.nom-4.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_integer__0_1_39",
        url = "https://crates.io/api/v1/crates/num-integer/0.1.39/download",
        type = "tar.gz",
        sha256 = "e83d528d2677f0518c570baf2b7abdcf0cd2d248860b68507bdcb3e91d4c0cea",
        strip_prefix = "num-integer-0.1.39",
        build_file = Label("//bazel/remote:BUILD.num-integer-0.1.39.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_traits__0_2_6",
        url = "https://crates.io/api/v1/crates/num-traits/0.2.6/download",
        type = "tar.gz",
        sha256 = "0b3a5d7cc97d6d30d8b9bc8fa19bf45349ffe46241e8816f50f62f6d6aaabee1",
        strip_prefix = "num-traits-0.2.6",
        build_file = Label("//bazel/remote:BUILD.num-traits-0.2.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__num_cpus__1_10_0",
        url = "https://crates.io/api/v1/crates/num_cpus/1.10.0/download",
        type = "tar.gz",
        sha256 = "1a23f0ed30a54abaa0c7e83b1d2d87ada7c3c23078d1d87815af3e3b6385fbba",
        strip_prefix = "num_cpus-1.10.0",
        build_file = Label("//bazel/remote:BUILD.num_cpus-1.10.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl__0_10_16",
        url = "https://crates.io/api/v1/crates/openssl/0.10.16/download",
        type = "tar.gz",
        sha256 = "ec7bd7ca4cce6dbdc77e7c1230682740d307d1218a87fb0349a571272be749f9",
        strip_prefix = "openssl-0.10.16",
        build_file = Label("//bazel/remote:BUILD.openssl-0.10.16.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl_probe__0_1_2",
        url = "https://crates.io/api/v1/crates/openssl-probe/0.1.2/download",
        type = "tar.gz",
        sha256 = "77af24da69f9d9341038eba93a073b1fdaaa1b788221b00a69bce9e762cb32de",
        strip_prefix = "openssl-probe-0.1.2",
        build_file = Label("//bazel/remote:BUILD.openssl-probe-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__openssl_sys__0_9_40",
        url = "https://crates.io/api/v1/crates/openssl-sys/0.9.40/download",
        type = "tar.gz",
        sha256 = "1bb974e77de925ef426b6bc82fce15fd45bdcbeb5728bffcfc7cdeeb7ce1c2d6",
        strip_prefix = "openssl-sys-0.9.40",
        build_file = Label("//bazel/remote:BUILD.openssl-sys-0.9.40.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ordered_float__1_0_1",
        url = "https://crates.io/api/v1/crates/ordered-float/1.0.1/download",
        type = "tar.gz",
        sha256 = "2f0015e9e8e28ee20c581cfbfe47c650cedeb9ed0721090e0b7ebb10b9cdbcc2",
        strip_prefix = "ordered-float-1.0.1",
        build_file = Label("//bazel/remote:BUILD.ordered-float-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__owning_ref__0_4_0",
        url = "https://crates.io/api/v1/crates/owning_ref/0.4.0/download",
        type = "tar.gz",
        sha256 = "49a4b8ea2179e6a2e27411d3bca09ca6dd630821cf6894c6c7c8467a8ee7ef13",
        strip_prefix = "owning_ref-0.4.0",
        build_file = Label("//bazel/remote:BUILD.owning_ref-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__parking_lot__0_7_1",
        url = "https://crates.io/api/v1/crates/parking_lot/0.7.1/download",
        type = "tar.gz",
        sha256 = "ab41b4aed082705d1056416ae4468b6ea99d52599ecf3169b00088d43113e337",
        strip_prefix = "parking_lot-0.7.1",
        build_file = Label("//bazel/remote:BUILD.parking_lot-0.7.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__parking_lot_core__0_4_0",
        url = "https://crates.io/api/v1/crates/parking_lot_core/0.4.0/download",
        type = "tar.gz",
        sha256 = "94c8c7923936b28d546dfd14d4472eaf34c99b14e1c973a32b3e6d4eb04298c9",
        strip_prefix = "parking_lot_core-0.4.0",
        build_file = Label("//bazel/remote:BUILD.parking_lot_core-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__percent_encoding__1_0_1",
        url = "https://crates.io/api/v1/crates/percent-encoding/1.0.1/download",
        type = "tar.gz",
        sha256 = "31010dd2e1ac33d5b46a5b413495239882813e0369f8ed8a5e266f173602f831",
        strip_prefix = "percent-encoding-1.0.1",
        build_file = Label("//bazel/remote:BUILD.percent-encoding-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__phf__0_7_24",
        url = "https://crates.io/api/v1/crates/phf/0.7.24/download",
        type = "tar.gz",
        sha256 = "b3da44b85f8e8dfaec21adae67f95d93244b2ecf6ad2a692320598dcc8e6dd18",
        strip_prefix = "phf-0.7.24",
        build_file = Label("//bazel/remote:BUILD.phf-0.7.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__phf_codegen__0_7_24",
        url = "https://crates.io/api/v1/crates/phf_codegen/0.7.24/download",
        type = "tar.gz",
        sha256 = "b03e85129e324ad4166b06b2c7491ae27fe3ec353af72e72cd1654c7225d517e",
        strip_prefix = "phf_codegen-0.7.24",
        build_file = Label("//bazel/remote:BUILD.phf_codegen-0.7.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__phf_generator__0_7_24",
        url = "https://crates.io/api/v1/crates/phf_generator/0.7.24/download",
        type = "tar.gz",
        sha256 = "09364cc93c159b8b06b1f4dd8a4398984503483891b0c26b867cf431fb132662",
        strip_prefix = "phf_generator-0.7.24",
        build_file = Label("//bazel/remote:BUILD.phf_generator-0.7.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__phf_shared__0_7_24",
        url = "https://crates.io/api/v1/crates/phf_shared/0.7.24/download",
        type = "tar.gz",
        sha256 = "234f71a15de2288bcb7e3b6515828d22af7ec8598ee6d24c3b526fa0a80b67a0",
        strip_prefix = "phf_shared-0.7.24",
        build_file = Label("//bazel/remote:BUILD.phf_shared-0.7.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pkg_config__0_3_14",
        url = "https://crates.io/api/v1/crates/pkg-config/0.3.14/download",
        type = "tar.gz",
        sha256 = "676e8eb2b1b4c9043511a9b7bea0915320d7e502b0a079fb03f9635a5252b18c",
        strip_prefix = "pkg-config-0.3.14",
        build_file = Label("//bazel/remote:BUILD.pkg-config-0.3.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__postgres__0_15_2",
        url = "https://crates.io/api/v1/crates/postgres/0.15.2/download",
        type = "tar.gz",
        sha256 = "115dde90ef51af573580c035857badbece2aa5cde3de1dfb3c932969ca92a6c5",
        strip_prefix = "postgres-0.15.2",
        build_file = Label("//bazel/remote:BUILD.postgres-0.15.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__postgres_protocol__0_3_2",
        url = "https://crates.io/api/v1/crates/postgres-protocol/0.3.2/download",
        type = "tar.gz",
        sha256 = "2487e66455bf88a1b247bf08a3ce7fe5197ac6d67228d920b0ee6a0e97fd7312",
        strip_prefix = "postgres-protocol-0.3.2",
        build_file = Label("//bazel/remote:BUILD.postgres-protocol-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__postgres_shared__0_4_2",
        url = "https://crates.io/api/v1/crates/postgres-shared/0.4.2/download",
        type = "tar.gz",
        sha256 = "ffac35b3e0029b404c24a3b82149b4e904f293e8ca4a327eefa24d3ca50df36f",
        strip_prefix = "postgres-shared-0.4.2",
        build_file = Label("//bazel/remote:BUILD.postgres-shared-0.4.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__pretty_assertions__0_5_1",
        url = "https://crates.io/api/v1/crates/pretty_assertions/0.5.1/download",
        type = "tar.gz",
        sha256 = "3a029430f0d744bc3d15dd474d591bed2402b645d024583082b9f63bb936dac6",
        strip_prefix = "pretty_assertions-0.5.1",
        build_file = Label("//bazel/remote:BUILD.pretty_assertions-0.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro2__0_4_27",
        url = "https://crates.io/api/v1/crates/proc-macro2/0.4.27/download",
        type = "tar.gz",
        sha256 = "4d317f9caece796be1980837fd5cb3dfec5613ebdb04ad0956deea83ce168915",
        strip_prefix = "proc-macro2-0.4.27",
        build_file = Label("//bazel/remote:BUILD.proc-macro2-0.4.27.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__proc_macro2__1_0_24",
        url = "https://crates.io/api/v1/crates/proc-macro2/1.0.24/download",
        type = "tar.gz",
        sha256 = "1e0704ee1a7e00d7bb417d0770ea303c1bccbabf0ef1667dae92b5967f5f8a71",
        strip_prefix = "proc-macro2-1.0.24",
        build_file = Label("//bazel/remote:BUILD.proc-macro2-1.0.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quick_error__1_2_2",
        url = "https://crates.io/api/v1/crates/quick-error/1.2.2/download",
        type = "tar.gz",
        sha256 = "9274b940887ce9addde99c4eee6b5c44cc494b182b97e73dc8ffdcb3397fd3f0",
        strip_prefix = "quick-error-1.2.2",
        build_file = Label("//bazel/remote:BUILD.quick-error-1.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quote__0_6_11",
        url = "https://crates.io/api/v1/crates/quote/0.6.11/download",
        type = "tar.gz",
        sha256 = "cdd8e04bd9c52e0342b406469d494fcb033be4bdbe5c606016defbb1681411e1",
        strip_prefix = "quote-0.6.11",
        build_file = Label("//bazel/remote:BUILD.quote-0.6.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__quote__1_0_9",
        url = "https://crates.io/api/v1/crates/quote/1.0.9/download",
        type = "tar.gz",
        sha256 = "c3d0b9745dc2debf507c8422de05d7226cc1f0644216dfdfead988f9b1ab32a7",
        strip_prefix = "quote-1.0.9",
        build_file = Label("//bazel/remote:BUILD.quote-1.0.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__r2d2__0_8_3",
        url = "https://crates.io/api/v1/crates/r2d2/0.8.3/download",
        type = "tar.gz",
        sha256 = "5d746fc8a0dab19ccea7ff73ad535854e90ddb3b4b8cdce953dd5cd0b2e7bd22",
        strip_prefix = "r2d2-0.8.3",
        build_file = Label("//bazel/remote:BUILD.r2d2-0.8.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__r2d2_postgres__0_14_0",
        url = "https://crates.io/api/v1/crates/r2d2_postgres/0.14.0/download",
        type = "tar.gz",
        sha256 = "78c7fe9c0c3d2c298cf262bc3ce4b89cdf0eab620fd9fe759f65b34a1a00fb93",
        strip_prefix = "r2d2_postgres-0.14.0",
        build_file = Label("//bazel/remote:BUILD.r2d2_postgres-0.14.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand__0_3_23",
        url = "https://crates.io/api/v1/crates/rand/0.3.23/download",
        type = "tar.gz",
        sha256 = "64ac302d8f83c0c1974bf758f6b041c6c8ada916fbb44a609158ca8b064cc76c",
        strip_prefix = "rand-0.3.23",
        build_file = Label("//bazel/remote:BUILD.rand-0.3.23.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand__0_4_6",
        url = "https://crates.io/api/v1/crates/rand/0.4.6/download",
        type = "tar.gz",
        sha256 = "552840b97013b1a26992c11eac34bdd778e464601a4c2054b5f0bff7c6761293",
        strip_prefix = "rand-0.4.6",
        build_file = Label("//bazel/remote:BUILD.rand-0.4.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand__0_5_6",
        url = "https://crates.io/api/v1/crates/rand/0.5.6/download",
        type = "tar.gz",
        sha256 = "c618c47cd3ebd209790115ab837de41425723956ad3ce2e6a7f09890947cacb9",
        strip_prefix = "rand-0.5.6",
        build_file = Label("//bazel/remote:BUILD.rand-0.5.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand__0_6_5",
        url = "https://crates.io/api/v1/crates/rand/0.6.5/download",
        type = "tar.gz",
        sha256 = "6d71dacdc3c88c1fde3885a3be3fbab9f35724e6ce99467f7d9c5026132184ca",
        strip_prefix = "rand-0.6.5",
        build_file = Label("//bazel/remote:BUILD.rand-0.6.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_chacha__0_1_1",
        url = "https://crates.io/api/v1/crates/rand_chacha/0.1.1/download",
        type = "tar.gz",
        sha256 = "556d3a1ca6600bfcbab7c7c91ccb085ac7fbbcd70e008a98742e7847f4f7bcef",
        strip_prefix = "rand_chacha-0.1.1",
        build_file = Label("//bazel/remote:BUILD.rand_chacha-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_core__0_3_1",
        url = "https://crates.io/api/v1/crates/rand_core/0.3.1/download",
        type = "tar.gz",
        sha256 = "7a6fdeb83b075e8266dcc8762c22776f6877a63111121f5f8c7411e5be7eed4b",
        strip_prefix = "rand_core-0.3.1",
        build_file = Label("//bazel/remote:BUILD.rand_core-0.3.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_core__0_4_0",
        url = "https://crates.io/api/v1/crates/rand_core/0.4.0/download",
        type = "tar.gz",
        sha256 = "d0e7a549d590831370895ab7ba4ea0c1b6b011d106b5ff2da6eee112615e6dc0",
        strip_prefix = "rand_core-0.4.0",
        build_file = Label("//bazel/remote:BUILD.rand_core-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_hc__0_1_0",
        url = "https://crates.io/api/v1/crates/rand_hc/0.1.0/download",
        type = "tar.gz",
        sha256 = "7b40677c7be09ae76218dc623efbf7b18e34bced3f38883af07bb75630a21bc4",
        strip_prefix = "rand_hc-0.1.0",
        build_file = Label("//bazel/remote:BUILD.rand_hc-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_isaac__0_1_1",
        url = "https://crates.io/api/v1/crates/rand_isaac/0.1.1/download",
        type = "tar.gz",
        sha256 = "ded997c9d5f13925be2a6fd7e66bf1872597f759fd9dd93513dd7e92e5a5ee08",
        strip_prefix = "rand_isaac-0.1.1",
        build_file = Label("//bazel/remote:BUILD.rand_isaac-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_jitter__0_1_3",
        url = "https://crates.io/api/v1/crates/rand_jitter/0.1.3/download",
        type = "tar.gz",
        sha256 = "7b9ea758282efe12823e0d952ddb269d2e1897227e464919a554f2a03ef1b832",
        strip_prefix = "rand_jitter-0.1.3",
        build_file = Label("//bazel/remote:BUILD.rand_jitter-0.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_os__0_1_2",
        url = "https://crates.io/api/v1/crates/rand_os/0.1.2/download",
        type = "tar.gz",
        sha256 = "b7c690732391ae0abafced5015ffb53656abfaec61b342290e5eb56b286a679d",
        strip_prefix = "rand_os-0.1.2",
        build_file = Label("//bazel/remote:BUILD.rand_os-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_pcg__0_1_1",
        url = "https://crates.io/api/v1/crates/rand_pcg/0.1.1/download",
        type = "tar.gz",
        sha256 = "086bd09a33c7044e56bb44d5bdde5a60e7f119a9e95b0775f545de759a32fe05",
        strip_prefix = "rand_pcg-0.1.1",
        build_file = Label("//bazel/remote:BUILD.rand_pcg-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rand_xorshift__0_1_1",
        url = "https://crates.io/api/v1/crates/rand_xorshift/0.1.1/download",
        type = "tar.gz",
        sha256 = "cbf7e9e623549b0e21f6e97cf8ecf247c1a8fd2e8a992ae265314300b2455d5c",
        strip_prefix = "rand_xorshift-0.1.1",
        build_file = Label("//bazel/remote:BUILD.rand_xorshift-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rdrand__0_4_0",
        url = "https://crates.io/api/v1/crates/rdrand/0.4.0/download",
        type = "tar.gz",
        sha256 = "678054eb77286b51581ba43620cc911abf02758c91f93f479767aed0f90458b2",
        strip_prefix = "rdrand-0.4.0",
        build_file = Label("//bazel/remote:BUILD.rdrand-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__redox_syscall__0_1_51",
        url = "https://crates.io/api/v1/crates/redox_syscall/0.1.51/download",
        type = "tar.gz",
        sha256 = "423e376fffca3dfa06c9e9790a9ccd282fafb3cc6e6397d01dbf64f9bacc6b85",
        strip_prefix = "redox_syscall-0.1.51",
        build_file = Label("//bazel/remote:BUILD.redox_syscall-0.1.51.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__redox_termios__0_1_1",
        url = "https://crates.io/api/v1/crates/redox_termios/0.1.1/download",
        type = "tar.gz",
        sha256 = "7e891cfe48e9100a70a3b6eb652fef28920c117d366339687bd5576160db0f76",
        strip_prefix = "redox_termios-0.1.1",
        build_file = Label("//bazel/remote:BUILD.redox_termios-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex__1_1_0",
        url = "https://crates.io/api/v1/crates/regex/1.1.0/download",
        type = "tar.gz",
        sha256 = "37e7cbbd370869ce2e8dff25c7018702d10b21a20ef7135316f8daecd6c25b7f",
        strip_prefix = "regex-1.1.0",
        build_file = Label("//bazel/remote:BUILD.regex-1.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__regex_syntax__0_6_5",
        url = "https://crates.io/api/v1/crates/regex-syntax/0.6.5/download",
        type = "tar.gz",
        sha256 = "8c2f35eedad5295fdf00a63d7d4b238135723f92b434ec06774dad15c7ab0861",
        strip_prefix = "regex-syntax-0.6.5",
        build_file = Label("//bazel/remote:BUILD.regex-syntax-0.6.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__remove_dir_all__0_5_1",
        url = "https://crates.io/api/v1/crates/remove_dir_all/0.5.1/download",
        type = "tar.gz",
        sha256 = "3488ba1b9a2084d38645c4c08276a1752dcbf2c7130d74f1569681ad5d2799c5",
        strip_prefix = "remove_dir_all-0.5.1",
        build_file = Label("//bazel/remote:BUILD.remove_dir_all-0.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__reqwest__0_9_9",
        url = "https://crates.io/api/v1/crates/reqwest/0.9.9/download",
        type = "tar.gz",
        sha256 = "09d6e187a58d923ee132fcda141c94e716bcfe301c2ea2bef5c81536e0085376",
        strip_prefix = "reqwest-0.9.9",
        build_file = Label("//bazel/remote:BUILD.reqwest-0.9.9.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__resolv_conf__0_6_2",
        url = "https://crates.io/api/v1/crates/resolv-conf/0.6.2/download",
        type = "tar.gz",
        sha256 = "b263b4aa1b5de9ffc0054a2386f96992058bb6870aab516f8cdeb8a667d56dcb",
        strip_prefix = "resolv-conf-0.6.2",
        build_file = Label("//bazel/remote:BUILD.resolv-conf-0.6.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__resolve__0_2_0",
        url = "https://crates.io/api/v1/crates/resolve/0.2.0/download",
        type = "tar.gz",
        sha256 = "19526b305899bea65f26edda78a64f5313958494321ee0ab66bd94b32958614a",
        strip_prefix = "resolve-0.2.0",
        build_file = Label("//bazel/remote:BUILD.resolve-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ring__0_13_5",
        url = "https://crates.io/api/v1/crates/ring/0.13.5/download",
        type = "tar.gz",
        sha256 = "2c4db68a2e35f3497146b7e4563df7d4773a2433230c5e4b448328e31740458a",
        strip_prefix = "ring-0.13.5",
        build_file = Label("//bazel/remote:BUILD.ring-0.13.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rle_decode_fast__1_0_1",
        url = "https://crates.io/api/v1/crates/rle-decode-fast/1.0.1/download",
        type = "tar.gz",
        sha256 = "cabe4fa914dec5870285fa7f71f602645da47c486e68486d2b4ceb4a343e90ac",
        strip_prefix = "rle-decode-fast-1.0.1",
        build_file = Label("//bazel/remote:BUILD.rle-decode-fast-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rustc_demangle__0_1_13",
        url = "https://crates.io/api/v1/crates/rustc-demangle/0.1.13/download",
        type = "tar.gz",
        sha256 = "adacaae16d02b6ec37fdc7acfcddf365978de76d1983d3ee22afc260e1ca9619",
        strip_prefix = "rustc-demangle-0.1.13",
        build_file = Label("//bazel/remote:BUILD.rustc-demangle-0.1.13.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__rustc_version__0_2_3",
        url = "https://crates.io/api/v1/crates/rustc_version/0.2.3/download",
        type = "tar.gz",
        sha256 = "138e3e0acb6c9fb258b19b67cb8abd63c00679d2851805ea151465464fe9030a",
        strip_prefix = "rustc_version-0.2.3",
        build_file = Label("//bazel/remote:BUILD.rustc_version-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ryu__0_2_7",
        url = "https://crates.io/api/v1/crates/ryu/0.2.7/download",
        type = "tar.gz",
        sha256 = "eb9e9b8cde282a9fe6a42dd4681319bfb63f121b8a8ee9439c6f4107e58a46f7",
        strip_prefix = "ryu-0.2.7",
        build_file = Label("//bazel/remote:BUILD.ryu-0.2.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__safemem__0_2_0",
        url = "https://crates.io/api/v1/crates/safemem/0.2.0/download",
        type = "tar.gz",
        sha256 = "e27a8b19b835f7aea908818e871f5cc3a5a186550c30773be987e155e8163d8f",
        strip_prefix = "safemem-0.2.0",
        build_file = Label("//bazel/remote:BUILD.safemem-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__safemem__0_3_0",
        url = "https://crates.io/api/v1/crates/safemem/0.3.0/download",
        type = "tar.gz",
        sha256 = "8dca453248a96cb0749e36ccdfe2b0b4e54a61bfef89fb97ec621eb8e0a93dd9",
        strip_prefix = "safemem-0.3.0",
        build_file = Label("//bazel/remote:BUILD.safemem-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__schannel__0_1_14",
        url = "https://crates.io/api/v1/crates/schannel/0.1.14/download",
        type = "tar.gz",
        sha256 = "0e1a231dc10abf6749cfa5d7767f25888d484201accbd919b66ab5413c502d56",
        strip_prefix = "schannel-0.1.14",
        build_file = Label("//bazel/remote:BUILD.schannel-0.1.14.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__scheduled_thread_pool__0_2_0",
        url = "https://crates.io/api/v1/crates/scheduled-thread-pool/0.2.0/download",
        type = "tar.gz",
        sha256 = "1a2ff3fc5223829be817806c6441279c676e454cc7da608faf03b0ccc09d3889",
        strip_prefix = "scheduled-thread-pool-0.2.0",
        build_file = Label("//bazel/remote:BUILD.scheduled-thread-pool-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__schemamama__0_3_0",
        url = "https://crates.io/api/v1/crates/schemamama/0.3.0/download",
        type = "tar.gz",
        sha256 = "1f726d3b10198a91b545c12e55775ddf4abb681056aa62adf75ed00b68855ef9",
        strip_prefix = "schemamama-0.3.0",
        build_file = Label("//bazel/remote:BUILD.schemamama-0.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__schemamama_postgres__0_2_3",
        url = "https://crates.io/api/v1/crates/schemamama_postgres/0.2.3/download",
        type = "tar.gz",
        sha256 = "9a69defe7b625fa5c4bfda0a1525c9729baef68f620e505464b7bf0a4d1697f6",
        strip_prefix = "schemamama_postgres-0.2.3",
        build_file = Label("//bazel/remote:BUILD.schemamama_postgres-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__scopeguard__0_3_3",
        url = "https://crates.io/api/v1/crates/scopeguard/0.3.3/download",
        type = "tar.gz",
        sha256 = "94258f53601af11e6a49f722422f6e3425c52b06245a5cf9bc09908b174f5e27",
        strip_prefix = "scopeguard-0.3.3",
        build_file = Label("//bazel/remote:BUILD.scopeguard-0.3.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__security_framework__0_2_2",
        url = "https://crates.io/api/v1/crates/security-framework/0.2.2/download",
        type = "tar.gz",
        sha256 = "bfab8dda0e7a327c696d893df9ffa19cadc4bd195797997f5223cf5831beaf05",
        strip_prefix = "security-framework-0.2.2",
        build_file = Label("//bazel/remote:BUILD.security-framework-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__security_framework_sys__0_2_3",
        url = "https://crates.io/api/v1/crates/security-framework-sys/0.2.3/download",
        type = "tar.gz",
        sha256 = "3d6696852716b589dff9e886ff83778bb635150168e83afa8ac6b8a78cb82abc",
        strip_prefix = "security-framework-sys-0.2.3",
        build_file = Label("//bazel/remote:BUILD.security-framework-sys-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__semver__0_9_0",
        url = "https://crates.io/api/v1/crates/semver/0.9.0/download",
        type = "tar.gz",
        sha256 = "1d7eb9ef2c18661902cc47e535f9bc51b78acd254da71d375c2f6720d9a40403",
        strip_prefix = "semver-0.9.0",
        build_file = Label("//bazel/remote:BUILD.semver-0.9.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__semver_parser__0_7_0",
        url = "https://crates.io/api/v1/crates/semver-parser/0.7.0/download",
        type = "tar.gz",
        sha256 = "388a1df253eca08550bef6c72392cfe7c30914bf41df5269b68cbd6ff8f570a3",
        strip_prefix = "semver-parser-0.7.0",
        build_file = Label("//bazel/remote:BUILD.semver-parser-0.7.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde__1_0_88",
        url = "https://crates.io/api/v1/crates/serde/1.0.88/download",
        type = "tar.gz",
        sha256 = "9f301d728f2b94c9a7691c90f07b0b4e8a4517181d9461be94c04bddeb4bd850",
        strip_prefix = "serde-1.0.88",
        build_file = Label("//bazel/remote:BUILD.serde-1.0.88.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_value__0_5_3",
        url = "https://crates.io/api/v1/crates/serde-value/0.5.3/download",
        type = "tar.gz",
        sha256 = "7a663f873dedc4eac1a559d4c6bc0d0b2c34dc5ac4702e105014b8281489e44f",
        strip_prefix = "serde-value-0.5.3",
        build_file = Label("//bazel/remote:BUILD.serde-value-0.5.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_derive__1_0_88",
        url = "https://crates.io/api/v1/crates/serde_derive/1.0.88/download",
        type = "tar.gz",
        sha256 = "beed18e6f5175aef3ba670e57c60ef3b1b74d250d962a26604bff4c80e970dd4",
        strip_prefix = "serde_derive-1.0.88",
        build_file = Label("//bazel/remote:BUILD.serde_derive-1.0.88.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_json__1_0_38",
        url = "https://crates.io/api/v1/crates/serde_json/1.0.38/download",
        type = "tar.gz",
        sha256 = "27dce848e7467aa0e2fcaf0a413641499c0b745452aaca1194d24dedde9e13c9",
        strip_prefix = "serde_json-1.0.38",
        build_file = Label("//bazel/remote:BUILD.serde_json-1.0.38.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_urlencoded__0_5_4",
        url = "https://crates.io/api/v1/crates/serde_urlencoded/0.5.4/download",
        type = "tar.gz",
        sha256 = "d48f9f99cd749a2de71d29da5f948de7f2764cc5a9d7f3c97e3514d4ee6eabf2",
        strip_prefix = "serde_urlencoded-0.5.4",
        build_file = Label("//bazel/remote:BUILD.serde_urlencoded-0.5.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serde_yaml__0_8_8",
        url = "https://crates.io/api/v1/crates/serde_yaml/0.8.8/download",
        type = "tar.gz",
        sha256 = "0887a8e097a69559b56aa2526bf7aff7c3048cf627dff781f0b56a6001534593",
        strip_prefix = "serde_yaml-0.8.8",
        build_file = Label("//bazel/remote:BUILD.serde_yaml-0.8.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serial_test__0_2_0",
        url = "https://crates.io/api/v1/crates/serial_test/0.2.0/download",
        type = "tar.gz",
        sha256 = "50bfbc39343545618d97869d77f38ed43e48dd77432717dbc7ed39d797f3ecbe",
        strip_prefix = "serial_test-0.2.0",
        build_file = Label("//bazel/remote:BUILD.serial_test-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__serial_test_derive__0_2_0",
        url = "https://crates.io/api/v1/crates/serial_test_derive/0.2.0/download",
        type = "tar.gz",
        sha256 = "89dd85be2e2ad75b041c9df2892ac078fa6e0b90024028b2b9fb4125b7530f01",
        strip_prefix = "serial_test_derive-0.2.0",
        build_file = Label("//bazel/remote:BUILD.serial_test_derive-0.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__sha1__0_6_0",
        url = "https://crates.io/api/v1/crates/sha1/0.6.0/download",
        type = "tar.gz",
        sha256 = "2579985fda508104f7587689507983eadd6a6e84dd35d6d115361f530916fa0d",
        strip_prefix = "sha1-0.6.0",
        build_file = Label("//bazel/remote:BUILD.sha1-0.6.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__sha2__0_7_1",
        url = "https://crates.io/api/v1/crates/sha2/0.7.1/download",
        type = "tar.gz",
        sha256 = "9eb6be24e4c23a84d7184280d2722f7f2731fcdd4a9d886efbfe4413e4847ea0",
        strip_prefix = "sha2-0.7.1",
        build_file = Label("//bazel/remote:BUILD.sha2-0.7.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__signal_hook__0_1_7",
        url = "https://crates.io/api/v1/crates/signal-hook/0.1.7/download",
        type = "tar.gz",
        sha256 = "1f272d1b7586bec132ed427f532dd418d8beca1ca7f2caf7df35569b1415a4b4",
        strip_prefix = "signal-hook-0.1.7",
        build_file = Label("//bazel/remote:BUILD.signal-hook-0.1.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__siphasher__0_2_3",
        url = "https://crates.io/api/v1/crates/siphasher/0.2.3/download",
        type = "tar.gz",
        sha256 = "0b8de496cf83d4ed58b6be86c3a275b8602f6ffe98d3024a869e124147a9a3ac",
        strip_prefix = "siphasher-0.2.3",
        build_file = Label("//bazel/remote:BUILD.siphasher-0.2.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__slab__0_4_2",
        url = "https://crates.io/api/v1/crates/slab/0.4.2/download",
        type = "tar.gz",
        sha256 = "c111b5bd5695e56cffe5129854aa230b39c93a305372fdbb2668ca2394eea9f8",
        strip_prefix = "slab-0.4.2",
        build_file = Label("//bazel/remote:BUILD.slab-0.4.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__smallvec__0_6_10",
        url = "https://crates.io/api/v1/crates/smallvec/0.6.10/download",
        type = "tar.gz",
        sha256 = "ab606a9c5e214920bb66c458cd7be8ef094f813f20fe77a54cc7dbfff220d4b7",
        strip_prefix = "smallvec-0.6.10",
        build_file = Label("//bazel/remote:BUILD.smallvec-0.6.10.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__socket2__0_3_8",
        url = "https://crates.io/api/v1/crates/socket2/0.3.8/download",
        type = "tar.gz",
        sha256 = "c4d11a52082057d87cb5caa31ad812f4504b97ab44732cd8359df2e9ff9f48e7",
        strip_prefix = "socket2-0.3.8",
        build_file = Label("//bazel/remote:BUILD.socket2-0.3.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__stable_deref_trait__1_1_1",
        url = "https://crates.io/api/v1/crates/stable_deref_trait/1.1.1/download",
        type = "tar.gz",
        sha256 = "dba1a27d3efae4351c8051072d619e3ade2820635c3958d826bfea39d59b54c8",
        strip_prefix = "stable_deref_trait-1.1.1",
        build_file = Label("//bazel/remote:BUILD.stable_deref_trait-1.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__string__0_1_3",
        url = "https://crates.io/api/v1/crates/string/0.1.3/download",
        type = "tar.gz",
        sha256 = "b639411d0b9c738748b5397d5ceba08e648f4f1992231aa859af1a017f31f60b",
        strip_prefix = "string-0.1.3",
        build_file = Label("//bazel/remote:BUILD.string-0.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__stringprep__0_1_2",
        url = "https://crates.io/api/v1/crates/stringprep/0.1.2/download",
        type = "tar.gz",
        sha256 = "8ee348cb74b87454fff4b551cbf727025810a004f88aeacae7f85b87f4e9a1c1",
        strip_prefix = "stringprep-0.1.2",
        build_file = Label("//bazel/remote:BUILD.stringprep-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__suppositions__0_1_4",
        url = "https://crates.io/api/v1/crates/suppositions/0.1.4/download",
        type = "tar.gz",
        sha256 = "15a79a4a46412182a639719257d8c11915e7bd69e9f6f499ac6ddf87e78d03d8",
        strip_prefix = "suppositions-0.1.4",
        build_file = Label("//bazel/remote:BUILD.suppositions-0.1.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__syn__0_15_26",
        url = "https://crates.io/api/v1/crates/syn/0.15.26/download",
        type = "tar.gz",
        sha256 = "f92e629aa1d9c827b2bb8297046c1ccffc57c99b947a680d3ccff1f136a3bee9",
        strip_prefix = "syn-0.15.26",
        build_file = Label("//bazel/remote:BUILD.syn-0.15.26.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__syn__1_0_60",
        url = "https://crates.io/api/v1/crates/syn/1.0.60/download",
        type = "tar.gz",
        sha256 = "c700597eca8a5a762beb35753ef6b94df201c81cca676604f547495a0d7f0081",
        strip_prefix = "syn-1.0.60",
        build_file = Label("//bazel/remote:BUILD.syn-1.0.60.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__synstructure__0_10_1",
        url = "https://crates.io/api/v1/crates/synstructure/0.10.1/download",
        type = "tar.gz",
        sha256 = "73687139bf99285483c96ac0add482c3776528beac1d97d444f6e91f203a2015",
        strip_prefix = "synstructure-0.10.1",
        build_file = Label("//bazel/remote:BUILD.synstructure-0.10.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__take_mut__0_2_2",
        url = "https://crates.io/api/v1/crates/take_mut/0.2.2/download",
        type = "tar.gz",
        sha256 = "f764005d11ee5f36500a149ace24e00e3da98b0158b3e2d53a7495660d3f4d60",
        strip_prefix = "take_mut-0.2.2",
        build_file = Label("//bazel/remote:BUILD.take_mut-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tempfile__3_0_7",
        url = "https://crates.io/api/v1/crates/tempfile/3.0.7/download",
        type = "tar.gz",
        sha256 = "b86c784c88d98c801132806dadd3819ed29d8600836c4088e855cdf3e178ed8a",
        strip_prefix = "tempfile-3.0.7",
        build_file = Label("//bazel/remote:BUILD.tempfile-3.0.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__termcolor__1_0_4",
        url = "https://crates.io/api/v1/crates/termcolor/1.0.4/download",
        type = "tar.gz",
        sha256 = "4096add70612622289f2fdcdbd5086dc81c1e2675e6ae58d6c4f62a16c6d7f2f",
        strip_prefix = "termcolor-1.0.4",
        build_file = Label("//bazel/remote:BUILD.termcolor-1.0.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__termion__1_5_1",
        url = "https://crates.io/api/v1/crates/termion/1.5.1/download",
        type = "tar.gz",
        sha256 = "689a3bdfaab439fd92bc87df5c4c78417d3cbe537487274e9b0b2dce76e92096",
        strip_prefix = "termion-1.5.1",
        build_file = Label("//bazel/remote:BUILD.termion-1.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thiserror__1_0_24",
        url = "https://crates.io/api/v1/crates/thiserror/1.0.24/download",
        type = "tar.gz",
        sha256 = "e0f4a65597094d4483ddaed134f409b2cb7c1beccf25201a9f73c719254fa98e",
        strip_prefix = "thiserror-1.0.24",
        build_file = Label("//bazel/remote:BUILD.thiserror-1.0.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thiserror_impl__1_0_24",
        url = "https://crates.io/api/v1/crates/thiserror-impl/1.0.24/download",
        type = "tar.gz",
        sha256 = "7765189610d8241a44529806d6fd1f2e0a08734313a35d5b3a556f92b381f3c0",
        strip_prefix = "thiserror-impl-1.0.24",
        build_file = Label("//bazel/remote:BUILD.thiserror-impl-1.0.24.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thread_id__3_3_0",
        url = "https://crates.io/api/v1/crates/thread-id/3.3.0/download",
        type = "tar.gz",
        sha256 = "c7fbf4c9d56b320106cd64fd024dadfa0be7cb4706725fc44a7d7ce952d820c1",
        strip_prefix = "thread-id-3.3.0",
        build_file = Label("//bazel/remote:BUILD.thread-id-3.3.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__thread_local__0_3_6",
        url = "https://crates.io/api/v1/crates/thread_local/0.3.6/download",
        type = "tar.gz",
        sha256 = "c6b53e329000edc2b34dbe8545fd20e55a333362d0a321909685a19bd28c3f1b",
        strip_prefix = "thread_local-0.3.6",
        build_file = Label("//bazel/remote:BUILD.thread_local-0.3.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__time__0_1_42",
        url = "https://crates.io/api/v1/crates/time/0.1.42/download",
        type = "tar.gz",
        sha256 = "db8dcfca086c1143c9270ac42a2bbd8a7ee477b78ac8e45b19abfb0cbede4b6f",
        strip_prefix = "time-0.1.42",
        build_file = Label("//bazel/remote:BUILD.time-0.1.42.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio__0_1_15",
        url = "https://crates.io/api/v1/crates/tokio/0.1.15/download",
        type = "tar.gz",
        sha256 = "e0500b88064f08bebddd0c0bed39e19f5c567a5f30975bee52b0c0d3e2eeb38c",
        strip_prefix = "tokio-0.1.15",
        build_file = Label("//bazel/remote:BUILD.tokio-0.1.15.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_codec__0_1_1",
        url = "https://crates.io/api/v1/crates/tokio-codec/0.1.1/download",
        type = "tar.gz",
        sha256 = "5c501eceaf96f0e1793cf26beb63da3d11c738c4a943fdf3746d81d64684c39f",
        strip_prefix = "tokio-codec-0.1.1",
        build_file = Label("//bazel/remote:BUILD.tokio-codec-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_current_thread__0_1_4",
        url = "https://crates.io/api/v1/crates/tokio-current-thread/0.1.4/download",
        type = "tar.gz",
        sha256 = "331c8acc267855ec06eb0c94618dcbbfea45bed2d20b77252940095273fb58f6",
        strip_prefix = "tokio-current-thread-0.1.4",
        build_file = Label("//bazel/remote:BUILD.tokio-current-thread-0.1.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_executor__0_1_6",
        url = "https://crates.io/api/v1/crates/tokio-executor/0.1.6/download",
        type = "tar.gz",
        sha256 = "30c6dbf2d1ad1de300b393910e8a3aa272b724a400b6531da03eed99e329fbf0",
        strip_prefix = "tokio-executor-0.1.6",
        build_file = Label("//bazel/remote:BUILD.tokio-executor-0.1.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_fs__0_1_5",
        url = "https://crates.io/api/v1/crates/tokio-fs/0.1.5/download",
        type = "tar.gz",
        sha256 = "0e9cbbc8a3698b7ab652340f46633364f9eaa928ddaaee79d8b8f356dd79a09d",
        strip_prefix = "tokio-fs-0.1.5",
        build_file = Label("//bazel/remote:BUILD.tokio-fs-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_io__0_1_11",
        url = "https://crates.io/api/v1/crates/tokio-io/0.1.11/download",
        type = "tar.gz",
        sha256 = "b53aeb9d3f5ccf2ebb29e19788f96987fa1355f8fe45ea193928eaaaf3ae820f",
        strip_prefix = "tokio-io-0.1.11",
        build_file = Label("//bazel/remote:BUILD.tokio-io-0.1.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_reactor__0_1_8",
        url = "https://crates.io/api/v1/crates/tokio-reactor/0.1.8/download",
        type = "tar.gz",
        sha256 = "afbcdb0f0d2a1e4c440af82d7bbf0bf91a8a8c0575bcd20c05d15be7e9d3a02f",
        strip_prefix = "tokio-reactor-0.1.8",
        build_file = Label("//bazel/remote:BUILD.tokio-reactor-0.1.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_signal__0_2_7",
        url = "https://crates.io/api/v1/crates/tokio-signal/0.2.7/download",
        type = "tar.gz",
        sha256 = "dd6dc5276ea05ce379a16de90083ec80836440d5ef8a6a39545a3207373b8296",
        strip_prefix = "tokio-signal-0.2.7",
        build_file = Label("//bazel/remote:BUILD.tokio-signal-0.2.7.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_sync__0_1_1",
        url = "https://crates.io/api/v1/crates/tokio-sync/0.1.1/download",
        type = "tar.gz",
        sha256 = "3742b64166c1ee9121f1921aea5a726098458926a6b732d906ef23b1f3ef6f4f",
        strip_prefix = "tokio-sync-0.1.1",
        build_file = Label("//bazel/remote:BUILD.tokio-sync-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_tcp__0_1_3",
        url = "https://crates.io/api/v1/crates/tokio-tcp/0.1.3/download",
        type = "tar.gz",
        sha256 = "1d14b10654be682ac43efee27401d792507e30fd8d26389e1da3b185de2e4119",
        strip_prefix = "tokio-tcp-0.1.3",
        build_file = Label("//bazel/remote:BUILD.tokio-tcp-0.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_threadpool__0_1_11",
        url = "https://crates.io/api/v1/crates/tokio-threadpool/0.1.11/download",
        type = "tar.gz",
        sha256 = "c3fd86cb15547d02daa2b21aadaf4e37dee3368df38a526178a5afa3c034d2fb",
        strip_prefix = "tokio-threadpool-0.1.11",
        build_file = Label("//bazel/remote:BUILD.tokio-threadpool-0.1.11.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_timer__0_2_10",
        url = "https://crates.io/api/v1/crates/tokio-timer/0.2.10/download",
        type = "tar.gz",
        sha256 = "2910970404ba6fa78c5539126a9ae2045d62e3713041e447f695f41405a120c6",
        strip_prefix = "tokio-timer-0.2.10",
        build_file = Label("//bazel/remote:BUILD.tokio-timer-0.2.10.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_udp__0_1_3",
        url = "https://crates.io/api/v1/crates/tokio-udp/0.1.3/download",
        type = "tar.gz",
        sha256 = "66268575b80f4a4a710ef83d087fdfeeabdce9b74c797535fbac18a2cb906e92",
        strip_prefix = "tokio-udp-0.1.3",
        build_file = Label("//bazel/remote:BUILD.tokio-udp-0.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tokio_uds__0_2_5",
        url = "https://crates.io/api/v1/crates/tokio-uds/0.2.5/download",
        type = "tar.gz",
        sha256 = "037ffc3ba0e12a0ab4aca92e5234e0dedeb48fddf6ccd260f1f150a36a9f2445",
        strip_prefix = "tokio-uds-0.2.5",
        build_file = Label("//bazel/remote:BUILD.tokio-uds-0.2.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__tower_service__0_1_0",
        url = "https://crates.io/api/v1/crates/tower-service/0.1.0/download",
        type = "tar.gz",
        sha256 = "b32f72af77f1bfe3d3d4da8516a238ebe7039b51dd8637a09841ac7f16d2c987",
        strip_prefix = "tower-service-0.1.0",
        build_file = Label("//bazel/remote:BUILD.tower-service-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__traitobject__0_1_0",
        url = "https://crates.io/api/v1/crates/traitobject/0.1.0/download",
        type = "tar.gz",
        sha256 = "efd1f82c56340fdf16f2a953d7bda4f8fdffba13d93b00844c25572110b26079",
        strip_prefix = "traitobject-0.1.0",
        build_file = Label("//bazel/remote:BUILD.traitobject-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__trust_dns_proto__0_5_0",
        url = "https://crates.io/api/v1/crates/trust-dns-proto/0.5.0/download",
        type = "tar.gz",
        sha256 = "0838272e89f1c693b4df38dc353412e389cf548ceed6f9fd1af5a8d6e0e7cf74",
        strip_prefix = "trust-dns-proto-0.5.0",
        build_file = Label("//bazel/remote:BUILD.trust-dns-proto-0.5.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__trust_dns_proto__0_6_3",
        url = "https://crates.io/api/v1/crates/trust-dns-proto/0.6.3/download",
        type = "tar.gz",
        sha256 = "09144f0992b0870fa8d2972cc069cbf1e3c0fda64d1f3d45c4d68d0e0b52ad4e",
        strip_prefix = "trust-dns-proto-0.6.3",
        build_file = Label("//bazel/remote:BUILD.trust-dns-proto-0.6.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__trust_dns_resolver__0_10_3",
        url = "https://crates.io/api/v1/crates/trust-dns-resolver/0.10.3/download",
        type = "tar.gz",
        sha256 = "8a9f877f7a1ad821ab350505e1f1b146a4960402991787191d6d8cab2ce2de2c",
        strip_prefix = "trust-dns-resolver-0.10.3",
        build_file = Label("//bazel/remote:BUILD.trust-dns-resolver-0.10.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__try_lock__0_2_2",
        url = "https://crates.io/api/v1/crates/try-lock/0.2.2/download",
        type = "tar.gz",
        sha256 = "e604eb7b43c06650e854be16a2a03155743d3752dd1c943f6829e26b7a36e382",
        strip_prefix = "try-lock-0.2.2",
        build_file = Label("//bazel/remote:BUILD.try-lock-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__typemap__0_3_3",
        url = "https://crates.io/api/v1/crates/typemap/0.3.3/download",
        type = "tar.gz",
        sha256 = "653be63c80a3296da5551e1bfd2cca35227e13cdd08c6668903ae2f4f77aa1f6",
        strip_prefix = "typemap-0.3.3",
        build_file = Label("//bazel/remote:BUILD.typemap-0.3.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__typenum__1_10_0",
        url = "https://crates.io/api/v1/crates/typenum/1.10.0/download",
        type = "tar.gz",
        sha256 = "612d636f949607bdf9b123b4a6f6d966dedf3ff669f7f045890d3a4a73948169",
        strip_prefix = "typenum-1.10.0",
        build_file = Label("//bazel/remote:BUILD.typenum-1.10.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ucd_util__0_1_3",
        url = "https://crates.io/api/v1/crates/ucd-util/0.1.3/download",
        type = "tar.gz",
        sha256 = "535c204ee4d8434478593480b8f86ab45ec9aae0e83c568ca81abf0fd0e88f86",
        strip_prefix = "ucd-util-0.1.3",
        build_file = Label("//bazel/remote:BUILD.ucd-util-0.1.3.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicase__1_4_2",
        url = "https://crates.io/api/v1/crates/unicase/1.4.2/download",
        type = "tar.gz",
        sha256 = "7f4765f83163b74f957c797ad9253caf97f103fb064d3999aea9568d09fc8a33",
        strip_prefix = "unicase-1.4.2",
        build_file = Label("//bazel/remote:BUILD.unicase-1.4.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicase__2_2_0",
        url = "https://crates.io/api/v1/crates/unicase/2.2.0/download",
        type = "tar.gz",
        sha256 = "9d3218ea14b4edcaccfa0df0a64a3792a2c32cc706f1b336e48867f9d3147f90",
        strip_prefix = "unicase-2.2.0",
        build_file = Label("//bazel/remote:BUILD.unicase-2.2.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_bidi__0_3_4",
        url = "https://crates.io/api/v1/crates/unicode-bidi/0.3.4/download",
        type = "tar.gz",
        sha256 = "49f2bd0c6468a8230e1db229cff8029217cf623c767ea5d60bfbd42729ea54d5",
        strip_prefix = "unicode-bidi-0.3.4",
        build_file = Label("//bazel/remote:BUILD.unicode-bidi-0.3.4.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_normalization__0_1_8",
        url = "https://crates.io/api/v1/crates/unicode-normalization/0.1.8/download",
        type = "tar.gz",
        sha256 = "141339a08b982d942be2ca06ff8b076563cbe223d1befd5450716790d44e2426",
        strip_prefix = "unicode-normalization-0.1.8",
        build_file = Label("//bazel/remote:BUILD.unicode-normalization-0.1.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_xid__0_1_0",
        url = "https://crates.io/api/v1/crates/unicode-xid/0.1.0/download",
        type = "tar.gz",
        sha256 = "fc72304796d0818e357ead4e000d19c9c174ab23dc11093ac919054d20a6a7fc",
        strip_prefix = "unicode-xid-0.1.0",
        build_file = Label("//bazel/remote:BUILD.unicode-xid-0.1.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unicode_xid__0_2_1",
        url = "https://crates.io/api/v1/crates/unicode-xid/0.2.1/download",
        type = "tar.gz",
        sha256 = "f7fe0bb3479651439c9112f72b6c505038574c9fbb575ed1bf3b797fa39dd564",
        strip_prefix = "unicode-xid-0.2.1",
        build_file = Label("//bazel/remote:BUILD.unicode-xid-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__unsafe_any__0_4_2",
        url = "https://crates.io/api/v1/crates/unsafe-any/0.4.2/download",
        type = "tar.gz",
        sha256 = "f30360d7979f5e9c6e6cea48af192ea8fab4afb3cf72597154b8f08935bc9c7f",
        strip_prefix = "unsafe-any-0.4.2",
        build_file = Label("//bazel/remote:BUILD.unsafe-any-0.4.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__untrusted__0_6_2",
        url = "https://crates.io/api/v1/crates/untrusted/0.6.2/download",
        type = "tar.gz",
        sha256 = "55cd1f4b4e96b46aeb8d4855db4a7a9bd96eeeb5c6a1ab54593328761642ce2f",
        strip_prefix = "untrusted-0.6.2",
        build_file = Label("//bazel/remote:BUILD.untrusted-0.6.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__url__1_7_2",
        url = "https://crates.io/api/v1/crates/url/1.7.2/download",
        type = "tar.gz",
        sha256 = "dd4e7c0d531266369519a4aa4f399d748bd37043b00bde1e4ff1f60a120b355a",
        strip_prefix = "url-1.7.2",
        build_file = Label("//bazel/remote:BUILD.url-1.7.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__utf8_ranges__1_0_2",
        url = "https://crates.io/api/v1/crates/utf8-ranges/1.0.2/download",
        type = "tar.gz",
        sha256 = "796f7e48bef87609f7ade7e06495a87d5cd06c7866e6a5cbfceffc558a243737",
        strip_prefix = "utf8-ranges-1.0.2",
        build_file = Label("//bazel/remote:BUILD.utf8-ranges-1.0.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__uuid__0_5_1",
        url = "https://crates.io/api/v1/crates/uuid/0.5.1/download",
        type = "tar.gz",
        sha256 = "bcc7e3b898aa6f6c08e5295b6c89258d1331e9ac578cc992fb818759951bdc22",
        strip_prefix = "uuid-0.5.1",
        build_file = Label("//bazel/remote:BUILD.uuid-0.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__uuid__0_7_2",
        url = "https://crates.io/api/v1/crates/uuid/0.7.2/download",
        type = "tar.gz",
        sha256 = "0238db0c5b605dd1cf51de0f21766f97fba2645897024461d6a00c036819a768",
        strip_prefix = "uuid-0.7.2",
        build_file = Label("//bazel/remote:BUILD.uuid-0.7.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__v_escape__0_3_2",
        url = "https://crates.io/api/v1/crates/v_escape/0.3.2/download",
        type = "tar.gz",
        sha256 = "c8b50688edb86f4c092a1a9fe8bda004b0faa3197100897653809e97e09a2814",
        strip_prefix = "v_escape-0.3.2",
        build_file = Label("//bazel/remote:BUILD.v_escape-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__v_escape_derive__0_2_1",
        url = "https://crates.io/api/v1/crates/v_escape_derive/0.2.1/download",
        type = "tar.gz",
        sha256 = "7cd994c63b487fef7aad31e5394ec04b9e24de7b32ea5251c9fb499cd2cbf44c",
        strip_prefix = "v_escape_derive-0.2.1",
        build_file = Label("//bazel/remote:BUILD.v_escape_derive-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__v_htmlescape__0_3_2",
        url = "https://crates.io/api/v1/crates/v_htmlescape/0.3.2/download",
        type = "tar.gz",
        sha256 = "020cae817dc82693aa523f01087b291b1c7a9ac8cea5c12297963f21769fb27f",
        strip_prefix = "v_htmlescape-0.3.2",
        build_file = Label("//bazel/remote:BUILD.v_htmlescape-0.3.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__vcpkg__0_2_6",
        url = "https://crates.io/api/v1/crates/vcpkg/0.2.6/download",
        type = "tar.gz",
        sha256 = "def296d3eb3b12371b2c7d0e83bfe1403e4db2d7a0bba324a12b21c4ee13143d",
        strip_prefix = "vcpkg-0.2.6",
        build_file = Label("//bazel/remote:BUILD.vcpkg-0.2.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__version_check__0_1_5",
        url = "https://crates.io/api/v1/crates/version_check/0.1.5/download",
        type = "tar.gz",
        sha256 = "914b1a6776c4c929a602fafd8bc742e06365d4bcbe48c30f9cca5824f70dc9dd",
        strip_prefix = "version_check-0.1.5",
        build_file = Label("//bazel/remote:BUILD.version_check-0.1.5.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__want__0_0_6",
        url = "https://crates.io/api/v1/crates/want/0.0.6/download",
        type = "tar.gz",
        sha256 = "797464475f30ddb8830cc529aaaae648d581f99e2036a928877dfde027ddf6b3",
        strip_prefix = "want-0.0.6",
        build_file = Label("//bazel/remote:BUILD.want-0.0.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__widestring__0_2_2",
        url = "https://crates.io/api/v1/crates/widestring/0.2.2/download",
        type = "tar.gz",
        sha256 = "7157704c2e12e3d2189c507b7482c52820a16dfa4465ba91add92f266667cadb",
        strip_prefix = "widestring-0.2.2",
        build_file = Label("//bazel/remote:BUILD.widestring-0.2.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi__0_2_8",
        url = "https://crates.io/api/v1/crates/winapi/0.2.8/download",
        type = "tar.gz",
        sha256 = "167dc9d6949a9b857f3451275e911c3f44255842c1f7a76f33c55103a909087a",
        strip_prefix = "winapi-0.2.8",
        build_file = Label("//bazel/remote:BUILD.winapi-0.2.8.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi__0_3_6",
        url = "https://crates.io/api/v1/crates/winapi/0.3.6/download",
        type = "tar.gz",
        sha256 = "92c1eb33641e276cfa214a0522acad57be5c56b10cb348b3c5117db75f3ac4b0",
        strip_prefix = "winapi-0.3.6",
        build_file = Label("//bazel/remote:BUILD.winapi-0.3.6.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_build__0_1_1",
        url = "https://crates.io/api/v1/crates/winapi-build/0.1.1/download",
        type = "tar.gz",
        sha256 = "2d315eee3b34aca4797b2da6b13ed88266e6d612562a0c46390af8299fc699bc",
        strip_prefix = "winapi-build-0.1.1",
        build_file = Label("//bazel/remote:BUILD.winapi-build-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_i686_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-i686-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6",
        strip_prefix = "winapi-i686-pc-windows-gnu-0.4.0",
        build_file = Label("//bazel/remote:BUILD.winapi-i686-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_util__0_1_2",
        url = "https://crates.io/api/v1/crates/winapi-util/0.1.2/download",
        type = "tar.gz",
        sha256 = "7168bab6e1daee33b4557efd0e95d5ca70a03706d39fa5f3fe7a236f584b03c9",
        strip_prefix = "winapi-util-0.1.2",
        build_file = Label("//bazel/remote:BUILD.winapi-util-0.1.2.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winapi_x86_64_pc_windows_gnu__0_4_0",
        url = "https://crates.io/api/v1/crates/winapi-x86_64-pc-windows-gnu/0.4.0/download",
        type = "tar.gz",
        sha256 = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f",
        strip_prefix = "winapi-x86_64-pc-windows-gnu-0.4.0",
        build_file = Label("//bazel/remote:BUILD.winapi-x86_64-pc-windows-gnu-0.4.0.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__wincolor__1_0_1",
        url = "https://crates.io/api/v1/crates/wincolor/1.0.1/download",
        type = "tar.gz",
        sha256 = "561ed901ae465d6185fa7864d63fbd5720d0ef718366c9a4dc83cf6170d7e9ba",
        strip_prefix = "wincolor-1.0.1",
        build_file = Label("//bazel/remote:BUILD.wincolor-1.0.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winreg__0_5_1",
        url = "https://crates.io/api/v1/crates/winreg/0.5.1/download",
        type = "tar.gz",
        sha256 = "a27a759395c1195c4cc5cda607ef6f8f6498f64e78f7900f5de0a127a424704a",
        strip_prefix = "winreg-0.5.1",
        build_file = Label("//bazel/remote:BUILD.winreg-0.5.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__winutil__0_1_1",
        url = "https://crates.io/api/v1/crates/winutil/0.1.1/download",
        type = "tar.gz",
        sha256 = "7daf138b6b14196e3830a588acf1e86966c694d3e8fb026fb105b8b5dca07e6e",
        strip_prefix = "winutil-0.1.1",
        build_file = Label("//bazel/remote:BUILD.winutil-0.1.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__ws2_32_sys__0_2_1",
        url = "https://crates.io/api/v1/crates/ws2_32-sys/0.2.1/download",
        type = "tar.gz",
        sha256 = "d59cefebd0c892fa2dd6de581e937301d8552cb44489cdff035c6187cb63fa5e",
        strip_prefix = "ws2_32-sys-0.2.1",
        build_file = Label("//bazel/remote:BUILD.ws2_32-sys-0.2.1.bazel"),
    )

    maybe(
        http_archive,
        name = "raze__yaml_rust__0_4_2",
        url = "https://crates.io/api/v1/crates/yaml-rust/0.4.2/download",
        type = "tar.gz",
        sha256 = "95acf0db5515d07da9965ec0e0ba6cc2d825e2caeb7303b66ca441729801254e",
        strip_prefix = "yaml-rust-0.4.2",
        build_file = Label("//bazel/remote:BUILD.yaml-rust-0.4.2.bazel"),
    )
