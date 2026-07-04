//! Compiles proto/*.proto into Rust at build time via prost-build.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(
        &[
            "proto/feature_record.proto",
            "proto/prediction_record.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
