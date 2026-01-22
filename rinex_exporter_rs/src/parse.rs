use rinex::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

/// Signal-to-Noise Ratio range in dB-Hz
#[derive(Debug, Serialize)]
pub struct SnrRange {
    /// Minimum SNR in dB-Hz
    pub min: f64,
    /// Maximum SNR in dB-Hz (infinity for unbounded)
    pub max: f64,
}

/// Pseudorange measurement (code observable)
#[derive(Debug, Serialize)]
pub struct Pseudorange {
    /// Distance measurement in meters
    pub meters: f64,
    /// Signal band and tracking mode (e.g., "1C", "2P", "5Q")
    pub signal: String,
}

/// Carrier phase measurement
#[derive(Debug, Serialize)]
pub struct CarrierPhase {
    /// Phase measurement in cycles
    pub cycles: f64,
    /// Signal band and tracking mode (e.g., "1C", "2P", "5Q")
    pub signal: String,
}

/// Doppler shift measurement
#[derive(Debug, Serialize)]
pub struct Doppler {
    /// Doppler shift in Hertz
    pub hertz: f64,
    /// Signal band and tracking mode (e.g., "1C", "2P", "5Q")
    pub signal: String,
}

/// Signal strength measurement
#[derive(Debug, Serialize)]
pub struct SignalStrength {
    /// Signal strength in dB-Hz
    pub db_hz: f64,
    /// Signal band and tracking mode (e.g., "1C", "2P", "5Q")
    pub signal: String,
}

/// Typed measurement based on observable code
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Measurement {
    Pseudorange(Pseudorange),
    CarrierPhase(CarrierPhase),
    Doppler(Doppler),
    SignalStrength(SignalStrength),
}

/// A timestamped observation data point
#[derive(Debug, Serialize)]
pub struct ObservationDataPoint {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Epoch in seconds since reference
    pub epoch_seconds: f64,
    /// Satellite identifier (e.g., "G01", "E05", "C14")
    pub satellite_id: String,
    /// GNSS constellation (GPS, Galileo, BeiDou, GLONASS, etc.)
    pub constellation: String,
    /// The measurement with appropriate type and units
    pub measurement: Measurement,
    /// Loss of Lock Indicator (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loss_of_lock_indicator: Option<u8>,
    /// Signal-to-Noise Ratio range in dB-Hz (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_to_noise_ratio: Option<SnrRange>,
}

/// A timestamped navigation/ephemeris data point
#[derive(Debug, Serialize)]
pub struct NavigationDataPoint {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Epoch in seconds since reference
    pub epoch_seconds: f64,
    /// Satellite identifier
    pub satellite_id: String,
    /// GNSS constellation
    pub constellation: String,
    /// Clock bias in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clock_bias: Option<f64>,
    /// Clock drift in seconds/second
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clock_drift: Option<f64>,
    /// Clock drift rate in seconds/second²
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clock_drift_rate: Option<f64>,
    /// Additional ephemeris parameters as key-value pairs
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub ephemeris: HashMap<String, f64>,
}

/// Combined output containing all parsed data points
#[derive(Debug, Serialize)]
pub struct RinexDataPoints {
    /// Source file name
    pub source_file: String,
    /// RINEX type (Observation or Navigation)
    pub rinex_type: String,
    /// Station name from header
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station: Option<String>,
    /// First epoch timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_epoch: Option<String>,
    /// Last epoch timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_epoch: Option<String>,
    /// Number of data points
    pub data_point_count: usize,
    /// Observation data points (for OBS files)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<ObservationDataPoint>,
    /// Navigation data points (for NAV files)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub navigation: Vec<NavigationDataPoint>,
}

fn epoch_to_iso8601(epoch: &Epoch) -> String {
    // Format epoch as ISO 8601
    format!("{}", epoch)
}

fn epoch_to_seconds(epoch: &Epoch) -> f64 {
    // Get seconds since J2000 epoch
    epoch.to_gpst_seconds()
}

fn constellation_name(constellation: &Constellation) -> String {
    match constellation {
        Constellation::GPS => "GPS".to_string(),
        Constellation::Glonass => "GLONASS".to_string(),
        Constellation::Galileo => "Galileo".to_string(),
        Constellation::BeiDou => "BeiDou".to_string(),
        Constellation::QZSS => "QZSS".to_string(),
        Constellation::IRNSS => "IRNSS".to_string(),
        Constellation::SBAS => "SBAS".to_string(),
        Constellation::Mixed => "Mixed".to_string(),
        _ => format!("{:?}", constellation),
    }
}

/// Parse observable code (e.g., "C1C", "L2P", "D5Q", "S1C") into typed Measurement
fn parse_measurement(observable: &str, value: f64) -> Measurement {
    let obs_type = observable.chars().next().unwrap_or('?');
    let signal = observable.get(1..).unwrap_or("").to_string();

    match obs_type {
        'C' => Measurement::Pseudorange(Pseudorange { meters: value, signal }),
        'L' => Measurement::CarrierPhase(CarrierPhase { cycles: value, signal }),
        'D' => Measurement::Doppler(Doppler { hertz: value, signal }),
        'S' => Measurement::SignalStrength(SignalStrength { db_hz: value, signal }),
        _ => Measurement::Pseudorange(Pseudorange { meters: value, signal: observable.to_string() }),
    }
}

fn snr_to_range(snr: &rinex::observation::SNR) -> SnrRange {
    use rinex::observation::SNR;
    match snr {
        SNR::DbHz0 => SnrRange { min: 0.0, max: 0.0 },
        SNR::DbHz12 => SnrRange { min: 0.0, max: 12.0 },
        SNR::DbHz12_17 => SnrRange { min: 12.0, max: 17.0 },
        SNR::DbHz18_23 => SnrRange { min: 18.0, max: 23.0 },
        SNR::DbHz24_29 => SnrRange { min: 24.0, max: 29.0 },
        SNR::DbHz30_35 => SnrRange { min: 30.0, max: 35.0 },
        SNR::DbHz36_41 => SnrRange { min: 36.0, max: 41.0 },
        SNR::DbHz42_47 => SnrRange { min: 42.0, max: 47.0 },
        SNR::DbHz48_53 => SnrRange { min: 48.0, max: 53.0 },
        // For practical reasons 65 dB-Hz is chosen, as this is already 'lab setting' signal/noise ratio
        SNR::DbHz54 => SnrRange { min: 54.0, max: 65.0 },
    }
}

/// Parse observation RINEX file and extract timestamped data points
fn parse_observation_file(rinex: &Rinex, file_name: &str) -> RinexDataPoints {
    let mut observations = Vec::new();

    // Iterate over all observations using the correct API
    // observations_iter() returns (&ObsKey, &Observations)
    // ObsKey has: epoch, flag
    // Observations has: clock, signals (Vec<SignalObservation>)
    // SignalObservation has: sv, value, observable, lli, snr
    for (obs_key, obs_data) in rinex.observations_iter() {
        let timestamp = epoch_to_iso8601(&obs_key.epoch);
        let epoch_seconds = epoch_to_seconds(&obs_key.epoch);

        // Iterate over each signal observation
        for signal in &obs_data.signals {
            let sv_str = format!("{}", signal.sv);
            let constellation = constellation_name(&signal.sv.constellation);

            let observable_str = signal.observable.to_string();
            let obs_point = ObservationDataPoint {
                timestamp: timestamp.clone(),
                epoch_seconds,
                satellite_id: sv_str,
                constellation,
                measurement: parse_measurement(&observable_str, signal.value),
                loss_of_lock_indicator: signal.lli.map(|l| l.bits()),
                signal_to_noise_ratio: signal.snr.as_ref().map(snr_to_range),
            };
            observations.push(obs_point);
        }
    }

    let first_epoch = rinex.first_epoch().map(|e| epoch_to_iso8601(&e));
    let last_epoch = rinex.last_epoch().map(|e| epoch_to_iso8601(&e));
    let station = rinex.header.geodetic_marker.as_ref().map(|m| m.name.clone());

    RinexDataPoints {
        source_file: file_name.to_string(),
        rinex_type: "Observation".to_string(),
        station,
        first_epoch,
        last_epoch,
        data_point_count: observations.len(),
        observations,
        navigation: Vec::new(),
    }
}

/// Parse navigation RINEX file and extract timestamped ephemeris data
fn parse_navigation_file(rinex: &Rinex, file_name: &str) -> RinexDataPoints {
    let mut navigation = Vec::new();

    // Iterate over navigation ephemeris frames
    // nav_ephemeris_frames_iter() returns (&NavKey, &Ephemeris)
    // NavKey has: epoch, sv, msgtype, frmtype
    for (nav_key, ephemeris) in rinex.nav_ephemeris_frames_iter() {
        let timestamp = epoch_to_iso8601(&nav_key.epoch);
        let epoch_seconds = epoch_to_seconds(&nav_key.epoch);
        let sv_str = format!("{}", nav_key.sv);
        let constellation = constellation_name(&nav_key.sv.constellation);

        // Extract clock parameters
        let (clock_bias, clock_drift, clock_drift_rate) = ephemeris.sv_clock();

        // Build additional ephemeris parameters
        let mut ephemeris_params = HashMap::new();

        // Try to get orbital elements if available (only for MEO satellites)
        if let Some(kepler) = ephemeris.kepler() {
            ephemeris_params.insert("semi_major_axis".to_string(), kepler.a);
            ephemeris_params.insert("eccentricity".to_string(), kepler.e);
            ephemeris_params.insert("inclination".to_string(), kepler.i_0);
            ephemeris_params.insert("omega".to_string(), kepler.omega);
            ephemeris_params.insert("omega_0".to_string(), kepler.omega_0);
            ephemeris_params.insert("mean_anomaly".to_string(), kepler.m_0);
        }

        let nav_point = NavigationDataPoint {
            timestamp,
            epoch_seconds,
            satellite_id: sv_str,
            constellation,
            clock_bias: Some(clock_bias),
            clock_drift: Some(clock_drift),
            clock_drift_rate: Some(clock_drift_rate),
            ephemeris: ephemeris_params,
        };
        navigation.push(nav_point);
    }

    let first_epoch = rinex.first_epoch().map(|e| epoch_to_iso8601(&e));
    let last_epoch = rinex.last_epoch().map(|e| epoch_to_iso8601(&e));

    RinexDataPoints {
        source_file: file_name.to_string(),
        rinex_type: "Navigation".to_string(),
        station: None,
        first_epoch,
        last_epoch,
        data_point_count: navigation.len(),
        observations: Vec::new(),
        navigation,
    }
}

/// Parse a RINEX file and return timestamped data points as JSON
pub fn parse_rinex_file(file_path: &Path) -> anyhow::Result<RinexDataPoints> {
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    info!("Parsing RINEX file: {}", file_path.display());

    let rinex = Rinex::from_file(file_path)?;

    let data_points = if rinex.is_observation_rinex() {
        debug!("Detected Observation RINEX file");
        parse_observation_file(&rinex, file_name)
    } else if rinex.is_navigation_rinex() {
        debug!("Detected Navigation RINEX file");
        parse_navigation_file(&rinex, file_name)
    } else {
        warn!("Unsupported RINEX type, attempting observation parse");
        parse_observation_file(&rinex, file_name)
    };

    info!(
        "Parsed {} data points from {}",
        data_points.data_point_count, file_name
    );

    Ok(data_points)
}

/// Parse all RINEX files in a directory and output JSON
pub fn parse_rinex_data(data_dir: &str) -> anyhow::Result<()> {
    let uncompressed_dir = format!("{}/uncompressed", data_dir);
    let json_dir = format!("{}/json", data_dir);
    std::fs::create_dir_all(&json_dir)?;

    info!("Parsing RINEX files from: {}", uncompressed_dir);

    for entry in std::fs::read_dir(&uncompressed_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip if not a file
        if !path.is_file() {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Parse navigation (.rnx) and observation (.crx) files
        let is_rinex = file_name.ends_with(".rnx") || file_name.ends_with(".crx");
        if !is_rinex {
            debug!("Skipping non-RINEX file: {}", file_name);
            continue;
        }

        // Check if JSON output already exists
        let json_file_name = format!("{}.json", file_name);
        let json_path = format!("{}/{}", json_dir, json_file_name);
        if Path::new(&json_path).exists() {
            debug!("Skipping {} - JSON already exists", file_name);
            continue;
        }

        match parse_rinex_file(&path) {
            Ok(data_points) => {
                // Write JSON output
                let json = serde_json::to_string_pretty(&data_points)?;
                std::fs::write(&json_path, json)?;
                info!("Wrote JSON output to: {}", json_path);
            }
            Err(e) => {
                warn!("Failed to parse {}: {}", file_name, e);
            }
        }
    }

    Ok(())
}
