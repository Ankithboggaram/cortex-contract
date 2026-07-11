//! Compiles ../proto/cortex/contract/v1/*.proto into Rust at build time via prost-build.
//!
//! The `../` is because `proto/` is language-neutral and lives at the repo
//! root, one level up from this crate (`rust/`), alongside `python/`.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(
        &[
            "../proto/cortex/contract/v1/feature_record.proto",
            "../proto/cortex/contract/v1/prediction_record.proto",
        ],
        &["../proto"],
    )?;
    Ok(())
}
