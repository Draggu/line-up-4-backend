use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let descriptor_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("descriptor.bin");

    tonic_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .build_client(false)
        .type_attribute("Player", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(&["schemas/schema.proto"], &["schemas/"])?;

    Ok(())
}
