fn main() {
    tonic_build::compile_protos("../../proto/harness.proto")
        .unwrap_or_else(|e| panic!("failed to compile harness.proto: {e}"));
}
