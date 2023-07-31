fn main() -> Result<(), Box<dyn std::error::Error>> {
    const OUT_DIR: &str = "src/pb";
    const API_NAME: &str = "reserv_api.proto";
    const API_ROOT: &str = "proto/";
    tonic_build::configure()
        .out_dir(OUT_DIR)
        .compile(&[API_NAME], &[API_ROOT])
        .unwrap();
    Ok(())
}
