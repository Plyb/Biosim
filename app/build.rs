use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("../shader", "spirv-unknown-vulkan1.1")
        .print_metadata(MetadataPrintout::Full)
        .shader_crate_features(if cfg!(feature = "rect_grid") { vec!["rect_grid".to_string()] } else { vec![] })
        .build()?;
    Ok(())
}