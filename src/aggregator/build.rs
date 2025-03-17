fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &[
                "proto/reply_server.proto",
                "proto/relationship_service.proto",
                "proto/post_server.proto",
                "proto/user_profile_service.proto",
                "proto/feed_service.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
