fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .compile(
            &[
                "api/proto/management/management.proto",
                "api/proto/backup/backup.proto",
            ],
            &["api/proto"],
        )?;
    Ok(())
}
