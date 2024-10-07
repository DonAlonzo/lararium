use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "proto")]
    {
        let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        tonic_build::configure()
            .file_descriptor_set_path(out_dir.join("descriptor.bin"))
            .extern_path(".lararium.types", "crate")
            .compile_protos(&["../../proto/lararium/services.proto"], &["../../proto"])?;
    }
    Ok(())
}
