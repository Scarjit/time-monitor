use crate::gps::{GPSSocket, SkyData, TpvData};
use crate::satellite::calculate_satellite_position;
use prometheus::{Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder};
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{debug, info, warn};

pub struct GPSMetricsCollector {
    gps_socket: GPSSocket,
    registry: Registry,

    // Connection metrics
    socket_connected: Gauge,

    // TPV metrics
    fix_status: Gauge,
    latitude: Gauge,
    longitude: Gauge,
    altitude_hae: Gauge,
    altitude_msl: Gauge,
    epx: Gauge,
    epy: Gauge,
    epv: Gauge,
    speed: Gauge,
    track: Gauge,
    climb: Gauge,

    // SKY metrics
    hdop: Gauge,
    vdop: Gauge,
    pdop: Gauge,
    gdop: Gauge,
    tdop: Gauge,
    satellites_visible: Gauge,
    satellites_used: Gauge,

    // Per-satellite metrics
    satellite_signal: GaugeVec,
    satellite_elevation: GaugeVec,
    satellite_azimuth: GaugeVec,
    satellite_used: GaugeVec,
    satellite_latitude: GaugeVec,
    satellite_longitude: GaugeVec,
}

impl GPSMetricsCollector {
    pub fn new(gps_address:SocketAddr) -> anyhow::Result<Self> {
        let registry = Registry::new();

        // Connection metrics
        let socket_connected = Gauge::new("gps_socket_connected", "GPS socket connection status (1=connected, 0=disconnected)")?;

        // Create TPV metrics
        let fix_status = Gauge::new("gps_fix_status", "GPS fix status (0=no fix, 2=2D, 3=3D)")?;
        let latitude = Gauge::new("gps_latitude", "GPS latitude in degrees")?;
        let longitude = Gauge::new("gps_longitude", "GPS longitude in degrees")?;
        let altitude_hae = Gauge::new("gps_altitude_hae", "GPS altitude HAE in meters")?;
        let altitude_msl = Gauge::new("gps_altitude_msl", "GPS altitude MSL in meters")?;
        let epx = Gauge::new("gps_epx", "Longitude error estimate in meters")?;
        let epy = Gauge::new("gps_epy", "Latitude error estimate in meters")?;
        let epv = Gauge::new("gps_epv", "Altitude error estimate in meters")?;
        let speed = Gauge::new("gps_speed", "GPS speed in m/s")?;
        let track = Gauge::new("gps_track", "GPS track in degrees")?;
        let climb = Gauge::new("gps_climb", "GPS climb rate in m/s")?;

        // Create SKY metrics
        let hdop = Gauge::new("gps_hdop", "Horizontal dilution of precision")?;
        let vdop = Gauge::new("gps_vdop", "Vertical dilution of precision")?;
        let pdop = Gauge::new("gps_pdop", "Position dilution of precision")?;
        let gdop = Gauge::new("gps_gdop", "Geometric dilution of precision")?;
        let tdop = Gauge::new("gps_tdop", "Time dilution of precision")?;
        let satellites_visible = Gauge::new("gps_satellites_visible", "Number of visible satellites")?;
        let satellites_used = Gauge::new("gps_satellites_used", "Number of satellites used in fix")?;

        // Per-satellite metrics
        let satellite_signal = GaugeVec::new(
            Opts::new("gps_satellite_signal_strength", "Satellite signal strength in dB-Hz"),
            &["prn", "gnss", "svid"]
        )?;
        let satellite_elevation = GaugeVec::new(
            Opts::new("gps_satellite_elevation", "Satellite elevation in degrees"),
            &["prn", "gnss", "svid"]
        )?;
        let satellite_azimuth = GaugeVec::new(
            Opts::new("gps_satellite_azimuth", "Satellite azimuth in degrees"),
            &["prn", "gnss", "svid"]
        )?;
        let satellite_used = GaugeVec::new(
            Opts::new("gps_satellite_used", "Whether satellite is used in fix (1=yes, 0=no)"),
            &["prn", "gnss", "svid"]
        )?;
        let satellite_latitude = GaugeVec::new(
            Opts::new("gps_satellite_latitude", "Approximate satellite ground position latitude"),
            &["prn", "gnss", "svid"]
        )?;
        let satellite_longitude = GaugeVec::new(
            Opts::new("gps_satellite_longitude", "Approximate satellite ground position longitude"),
            &["prn", "gnss", "svid"]
        )?;

        // Register all metrics
        registry.register(Box::new(socket_connected.clone()))?;
        registry.register(Box::new(fix_status.clone()))?;
        registry.register(Box::new(latitude.clone()))?;
        registry.register(Box::new(longitude.clone()))?;
        registry.register(Box::new(altitude_hae.clone()))?;
        registry.register(Box::new(altitude_msl.clone()))?;
        registry.register(Box::new(epx.clone()))?;
        registry.register(Box::new(epy.clone()))?;
        registry.register(Box::new(epv.clone()))?;
        registry.register(Box::new(speed.clone()))?;
        registry.register(Box::new(track.clone()))?;
        registry.register(Box::new(climb.clone()))?;
        registry.register(Box::new(hdop.clone()))?;
        registry.register(Box::new(vdop.clone()))?;
        registry.register(Box::new(pdop.clone()))?;
        registry.register(Box::new(gdop.clone()))?;
        registry.register(Box::new(tdop.clone()))?;
        registry.register(Box::new(satellites_visible.clone()))?;
        registry.register(Box::new(satellites_used.clone()))?;
        registry.register(Box::new(satellite_signal.clone()))?;
        registry.register(Box::new(satellite_elevation.clone()))?;
        registry.register(Box::new(satellite_azimuth.clone()))?;
        registry.register(Box::new(satellite_used.clone()))?;
        registry.register(Box::new(satellite_latitude.clone()))?;
        registry.register(Box::new(satellite_longitude.clone()))?;

        Ok(Self {
            gps_socket: GPSSocket::new(gps_address),
            registry,
            socket_connected,
            fix_status,
            latitude,
            longitude,
            altitude_hae,
            altitude_msl,
            epx,
            epy,
            epv,
            speed,
            track,
            climb,
            hdop,
            vdop,
            pdop,
            gdop,
            tdop,
            satellites_visible,
            satellites_used,
            satellite_signal,
            satellite_elevation,
            satellite_azimuth,
            satellite_used,
            satellite_latitude,
            satellite_longitude,
        })
    }

    pub fn update_metrics(&mut self) -> anyhow::Result<()> {
        debug!("Updating GPS metrics");

        // Try to connect if not connected
        if let Err(e) = self.gps_socket.connect() {
            warn!("Failed to connect to GPS socket: {}", e);
            self.socket_connected.set(0.0);
            return Ok(()); // Silently fail, will retry next time
        }

        // Try to read GPS data
        match self.gps_socket.read_gps_data(Duration::from_secs(5)) {
            Ok((tpv_data, sky_data)) => {
                self.socket_connected.set(1.0);
                debug!("Successfully read GPS data");

                if let Some(ref tpv) = tpv_data {
                    debug!("Updating TPV metrics (mode={}, lat={:?}, lon={:?})",
                           tpv.mode, tpv.lat, tpv.lon);
                    self.update_tpv_metrics(tpv);
                }

                if let Some(ref sky) = sky_data {
                    debug!("Updating SKY metrics ({} visible, {} used satellites)",
                           sky.n_sat, sky.u_sat);
                    self.update_sky_metrics(sky, tpv_data.as_ref());
                }

                info!("Metrics updated successfully");
                Ok(())
            }
            Err(e) => {
                warn!("Failed to read GPS data: {}", e);
                self.socket_connected.set(0.0);
                // Disconnect so we can reconnect on next update
                self.gps_socket.disconnect();
                Err(e)
            }
        }
    }

    fn update_tpv_metrics(&self, tpv: &TpvData) {
        self.fix_status.set(tpv.mode as f64);

        if let Some(lat) = tpv.lat {
            self.latitude.set(lat);
        }
        if let Some(lon) = tpv.lon {
            self.longitude.set(lon);
        }
        if let Some(alt) = tpv.alt_hae {
            self.altitude_hae.set(alt);
        }
        if let Some(alt) = tpv.alt_msl {
            self.altitude_msl.set(alt);
        }
        if let Some(x) = tpv.epx {
            self.epx.set(x);
        }
        if let Some(y) = tpv.epy {
            self.epy.set(y);
        }
        if let Some(v) = tpv.epv {
            self.epv.set(v);
        }
        if let Some(s) = tpv.speed {
            self.speed.set(s);
        }
        if let Some(t) = tpv.track {
            self.track.set(t);
        }
        if let Some(c) = tpv.climb {
            self.climb.set(c);
        }
    }

    fn update_sky_metrics(&self, sky: &SkyData, tpv: Option<&TpvData>) {
        if let Some(h) = sky.hdop {
            self.hdop.set(h);
        }
        if let Some(v) = sky.vdop {
            self.vdop.set(v);
        }
        if let Some(p) = sky.pdop {
            self.pdop.set(p);
        }
        if let Some(g) = sky.gdop {
            self.gdop.set(g);
        }
        if let Some(t) = sky.tdop {
            self.tdop.set(t);
        }

        self.satellites_visible.set(sky.n_sat as f64);
        self.satellites_used.set(sky.u_sat as f64);

        // Clear old satellite metrics
        self.satellite_signal.reset();
        self.satellite_elevation.reset();
        self.satellite_azimuth.reset();
        self.satellite_used.reset();
        self.satellite_latitude.reset();
        self.satellite_longitude.reset();

        // Update per-satellite metrics
        if let Some(tpv_data) = tpv
            && let (Some(obs_lat), Some(obs_lon)) = (tpv_data.lat, tpv_data.lon) {
                for sat in &sky.satellites {
                    let prn = sat.prn.to_string();
                    let gnss = gnss_name(sat.gnssid);
                    let svid = sat.svid.to_string();
                    let labels = &[prn.as_str(), &gnss, svid.as_str()];

                    self.satellite_signal.with_label_values(labels).set(sat.ss);
                    self.satellite_elevation.with_label_values(labels).set(sat.el);
                    self.satellite_azimuth.with_label_values(labels).set(sat.az);
                    self.satellite_used.with_label_values(labels).set(if sat.used { 1.0 } else { 0.0 });

                    let (sat_lat, sat_lon) = calculate_satellite_position(obs_lat, obs_lon, sat.az, sat.el);
                    self.satellite_latitude.with_label_values(labels).set(sat_lat);
                    self.satellite_longitude.with_label_values(labels).set(sat_lon);
                }
            }
    }

    pub fn render_metrics(&self) -> anyhow::Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}

fn gnss_name(gnssid: i64) -> String {
    match gnssid {
        0 => "GPS".to_string(),
        1 => "SBAS".to_string(),
        2 => "Galileo".to_string(),
        3 => "BeiDou".to_string(),
        4 => "IMES".to_string(),
        5 => "QZSS".to_string(),
        6 => "GLONASS".to_string(),
        _ => format!("GNSS{}", gnssid),
    }
}
