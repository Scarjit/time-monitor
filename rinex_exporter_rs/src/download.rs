use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

pub async fn download_rinex_data(
    station: &str,
    backfill_days: u32,
    observation_duration: &str,
    observation_interval: &str,
    data_dir: &str,
) -> anyhow::Result<()> {
    let now = time::OffsetDateTime::now_utc();
    info!("Current date: {}", now.date());
    info!("Current time (UTC): {}", now.time());

    let current_year = now.year();
    let current_day_of_year = now.date().ordinal();
    let start_day_of_year = current_day_of_year - backfill_days as u16;

    for day_of_year in start_day_of_year..current_day_of_year {
        download_day(
            station,
            current_year,
            day_of_year,
            observation_duration,
            observation_interval,
            data_dir,
        )
        .await?;
    }

    info!("Downloaded all RINEX files");

    Ok(())
}

async fn download_day(
    station: &str,
    year: i32,
    day_of_year: u16,
    observation_duration: &str,
    observation_interval: &str,
    data_dir: &str,
) -> anyhow::Result<()> {
    let day_of_year_zero_prefixed = format!("{:03}", day_of_year);
    let base_url = format!(
        "https://igs.bkg.bund.de/root_ftp/EUREF/obs/{}/{}",
        year, day_of_year_zero_prefixed
    );

    let mo_filename = format!(
        "{}_R_{}{}0000_{}_{}_MO.crx.gz",
        station, year, day_of_year_zero_prefixed, observation_duration, observation_interval
    );
    let nav_filename = format!(
        "{}_R_{}{}0000_{}_MN.rnx.gz",
        station, year, day_of_year_zero_prefixed, observation_duration
    );

    info!("Downloading {} and {}", mo_filename, nav_filename);
    download_file(&base_url, &mo_filename, data_dir).await?;
    download_file(&base_url, &nav_filename, data_dir).await?;
    Ok(())
}

async fn download_file(base_url: &str, file_name: &str, data_dir: &str) -> anyhow::Result<()> {
    let out_file_name = format!("{}/compressed/{}", data_dir, file_name);
    // Skip download if the file already exists
    if Path::new(&out_file_name).exists() {
        debug!("Skipping download of {} as it already exists", file_name);
        return Ok(());
    }
    // mkdir -p the compressed dir
    tokio::fs::create_dir_all(format!("{}/compressed", data_dir)).await?;

    let resp = reqwest::get(format!("{}/{}", base_url, file_name)).await?;
    let body_bytes = resp.bytes().await?;
    let mut out = File::create(out_file_name).await?;

    out.write_all(&body_bytes).await?;
    info!("Downloaded {}", file_name);
    Ok(())
}
