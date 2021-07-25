use std::io::Result;
fn main() -> Result<()> {
    tonic_build::configure()
        .build_server(true)
        .out_dir("src/")
        .compile(&["protos/api_v0_1_x.proto"], &["protos"])?;
    Ok(())
}