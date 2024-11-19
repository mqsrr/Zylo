fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .disable_package_emission()
        .compile_protos(&["protos/user-profile-service.proto"], &["protos"])?;
    Ok(())
}