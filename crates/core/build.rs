use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "proto")]
    {
        tonic_build::configure()
            .extern_path(".lararium.types", "crate")
            .compile_protos(&["../../proto/lararium/services.proto"], &["../../proto"])?;
    }
    Ok(())
}
