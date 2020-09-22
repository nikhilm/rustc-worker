load("@rules_proto//proto:defs.bzl", "proto_library")
load("@io_bazel_rules_rust//proto:proto.bzl", "rust_proto_library")
load("@io_bazel_rules_rust//proto:toolchain.bzl", "PROTO_COMPILE_DEPS", "rust_proto_toolchain")

rust_proto_toolchain(name = "default-proto-toolchain-impl")

toolchain(
    name = "default-proto-toolchain",
    toolchain = ":default-proto-toolchain-impl",
    toolchain_type = "@io_bazel_rules_rust//proto:toolchain",
)

proto_library(
    name = "worker_protocol_proto",
    srcs = ["src/worker_protocol.proto"],
)

rust_proto_library(
    name = "worker_protocol",
    deps = [":worker_protocol_proto"],
)
