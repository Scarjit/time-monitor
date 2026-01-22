use std::path::Path;
use tokio::fs::File;
use tracing::{debug, info};

pub async fn uncompress_rinex_data(data_dir: &str) -> anyhow::Result<()> {
    let compressed_dir = format!("{}/compressed", data_dir);
    let uncompressed_dir = format!("{}/uncompressed", data_dir);
    tokio::fs::create_dir_all(&compressed_dir).await?;
    tokio::fs::create_dir_all(&uncompressed_dir).await?;

    // For each compressed RINEX file, uncompress it and move it to the uncompressed dir
    for entry in std::fs::read_dir(&compressed_dir)? {
        let entry = entry?;
        let path = entry.path();

        // We first get the file without the .gz extension
        let file_stem = path
            .file_stem()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path: no stem"))?;
        let file_name = file_stem
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name: not valid UTF-8"))?;
        let out_file_name = format!("{}/{}", uncompressed_dir, file_name);

        info!("Uncompressing {} to {}", path.display(), out_file_name);
        // Check if the uncompressed file already exists
        if Path::new(&out_file_name).exists() {
            debug!(
                "Skipping uncompressing of {} as it already exists",
                file_name
            );
            continue;
        }
        let f = File::open(&path).await?;
        let mut g =
            async_compression::tokio::bufread::GzipDecoder::new(tokio::io::BufReader::new(f));
        let mut out = File::create(&out_file_name).await?;
        tokio::io::copy(&mut g, &mut out).await?;
    }

    Ok(())
}
