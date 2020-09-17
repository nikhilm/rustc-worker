fn main() {
    prost_build::compile_protos(&["src/worker_protocol.proto"], &["src/"]).unwrap();
}
