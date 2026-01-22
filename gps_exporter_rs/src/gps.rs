use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkySatellite {
    #[serde(rename = "PRN")]
    pub prn: i64,
    pub gnssid: i64,
    pub svid: i64,
    pub az: f64,
    pub el: f64,
    pub ss: f64,
    pub used: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkyData {
    pub class: String,
    pub device: String,
    pub gdop: Option<f64>,
    pub hdop: Option<f64>,
    pub pdop: Option<f64>,
    pub tdop: Option<f64>,
    pub xdop: Option<f64>,
    pub ydop: Option<f64>,
    pub vdop: Option<f64>,
    #[serde(rename = "nSat")]
    pub n_sat: i64,
    #[serde(rename = "uSat")]
    pub u_sat: i64,
    pub satellites: Vec<SkySatellite>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TpvData {
    pub class: String,
    pub device: String,
    pub mode: i64,
    pub time: Option<String>,
    pub leapseconds: Option<i64>,
    pub ept: Option<f64>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    #[serde(rename = "altHAE")]
    pub alt_hae: Option<f64>,
    #[serde(rename = "altMSL")]
    pub alt_msl: Option<f64>,
    pub alt: Option<f64>,
    pub epx: Option<f64>,
    pub epy: Option<f64>,
    pub epv: Option<f64>,
    pub magvar: Option<f64>,
    pub speed: Option<f64>,
    pub track: Option<f64>,
    pub climb: Option<f64>,
    pub eps: Option<f64>,
    pub epc: Option<f64>,
    pub geoid_sep: Option<f64>,
    pub eph: Option<f64>,
    pub sep: Option<f64>,
}

pub struct GPSSocket {
    address: SocketAddr,
    stream: Option<TcpStream>,
}

impl GPSSocket {
    pub fn new(address: SocketAddr) -> GPSSocket {
        GPSSocket {
            address,
            stream: None,
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        // Check if already connected
        if self.stream.is_some() {
            debug!("Already connected to gpsd");
            return Ok(());
        }

        debug!("Attempting to connect to gpsd at {}", self.address);
        let mut stream = TcpStream::connect_timeout(&self.address, Duration::from_millis(1000))?;
        debug!("TCP connection established, sending WATCH command");
        stream.write_all(b"?WATCH={\"enable\":true,\"json\":true}\n")?;
        self.stream = Some(stream);
        info!("Successfully connected to gpsd at {}", self.address);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        if self.stream.is_some() {
            debug!("Disconnecting from gpsd");
            self.stream = None;
        }
    }

    pub fn read_gps_data(&mut self, timeout: Duration) -> Result<(Option<TpvData>, Option<SkyData>)> {
        let stream = match &mut self.stream {
            Some(stream) => stream,
            None => anyhow::bail!("Not connected to GPS socket"),
        };
        stream.set_read_timeout(Some(timeout))?;

        debug!("Reading GPS data with timeout of {:?}", timeout);
        let end_time = Instant::now() + timeout;
        let mut buffer: Vec<u8> = Vec::new();
        let mut tpv_data: Option<TpvData> = None;
        let mut sky_data: Option<SkyData> = None;

        while Instant::now() < end_time {
            let mut chunk = vec![0u8; 1024];
            let bytes_read = stream.read(&mut chunk)?;
            if bytes_read == 0 {
                debug!("Connection closed by gpsd (0 bytes read)");
                break;
            }
            debug!("Read {} bytes from gpsd", bytes_read);
            // SAFETY: We are slicing the chunk to the number of bytes read, which is safe
            #[allow(clippy::indexing_slicing)]{
                buffer.extend_from_slice(&chunk[..bytes_read]);
            }

            let lines = buffer.split(|x| *x == b'\n').collect::<Vec<&[u8]>>();
            let incomplete_line = lines.last().map(|l| l.to_vec());

            let (new_tpv, new_sky) = Self::parse_gpsd_data(&lines);
            if new_tpv.is_some() {
                debug!("Received TPV data from gpsd");
                tpv_data = new_tpv;
            }
            if new_sky.is_some() {
                debug!("Received SKY data from gpsd");
                sky_data = new_sky;
            }

            match incomplete_line {
                None => {
                    buffer.clear();
                }
                Some(incomplete) => {
                    buffer = incomplete;
                }
            }

            if tpv_data.is_some() && sky_data.is_some() {
                info!("Successfully received both TPV and SKY data");
                break;
            }
        }

        if tpv_data.is_none() && sky_data.is_none() {
            warn!("No GPS data received within timeout");
        }

        Ok((tpv_data, sky_data))
    }

    fn parse_gpsd_data(lines: &[&[u8]]) -> (Option<TpvData>, Option<SkyData>) {
        let mut maybe_sky_data: Option<SkyData> = None;
        let mut maybe_tpv_data: Option<TpvData> = None;

        for line in lines {
            if line.len() < 14 {
                continue;
            }

            if line.starts_with(b"{\"class\":\"SKY\"") {
                if let Ok(sky_data) = serde_json::from_slice(line) {
                    maybe_sky_data = Some(sky_data);
                }
            } else if line.starts_with(b"{\"class\":\"TPV\"")
                && let Ok(tpv_data) = serde_json::from_slice(line) {
                    maybe_tpv_data = Some(tpv_data);
                }

            if maybe_sky_data.is_some() && maybe_tpv_data.is_some() {
                break;
            }
        }
        (maybe_tpv_data, maybe_sky_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tpv_data() {
        let line = br#"{"class":"TPV","device":"/dev/ttyAMA0","mode":3,"time":"2026-01-14T12:19:37.000Z","leapseconds":18,"ept":0.005,"lat":50.970930167,"lon":6.936929333,"altHAE":110.2000,"altMSL":63.6000,"alt":63.6000,"epx":0.900,"epy":1.300,"epv":2.100,"magvar":2.1,"speed":0.015,"climb":-0.100,"eps":2.60,"epc":4.20,"geoidSep":46.600,"eph":13.300,"sep":24.130}"#;
        let lines = vec![line.as_slice()];
        let (tpv_data, _) = GPSSocket::parse_gpsd_data(&lines);

        assert!(tpv_data.is_some());
        let tpv = tpv_data.unwrap();
        assert_eq!(tpv.mode, 3);
        assert_eq!(tpv.lat, Some(50.970930167));
        assert_eq!(tpv.lon, Some(6.936929333));
        assert_eq!(tpv.alt_hae, Some(110.2000));
        assert_eq!(tpv.speed, Some(0.015));
    }

    #[test]
    fn test_parse_sky_data() {
        let line = br#"{"class":"SKY","device":"/dev/ttyAMA0","gdop":1.79,"hdop":0.70,"pdop":1.27,"tdop":0.81,"xdop":0.54,"ydop":0.68,"vdop":1.06,"nSat":10,"uSat":9,"satellites":[{"PRN":6,"gnssid":0,"svid":6,"az":40.0,"el":19.0,"ss":31.0,"used":true},{"PRN":11,"gnssid":0,"svid":11,"az":86.0,"el":32.0,"ss":40.0,"used":true}]}"#;
        let lines = vec![line.as_slice()];
        let (_, sky_data) = GPSSocket::parse_gpsd_data(&lines);

        assert!(sky_data.is_some());
        let sky = sky_data.unwrap();
        assert_eq!(sky.n_sat, 10);
        assert_eq!(sky.u_sat, 9);
        assert_eq!(sky.hdop, Some(0.70));
        assert_eq!(sky.satellites.len(), 2);
        assert_eq!(sky.satellites[0].prn, 6);
        assert_eq!(sky.satellites[0].az, 40.0);
        assert_eq!(sky.satellites[0].el, 19.0);
        assert!(sky.satellites[0].used);
    }

    #[test]
    fn test_parse_both_tpv_and_sky() {
        let tpv_line = br#"{"class":"TPV","device":"/dev/ttyAMA0","mode":3,"time":"2026-01-14T12:19:37.000Z","leapseconds":18,"ept":0.005,"lat":50.970930167,"lon":6.936929333,"altHAE":110.2000,"altMSL":63.6000,"alt":63.6000,"epx":0.900,"epy":1.300,"epv":2.100,"magvar":2.1,"speed":0.015,"climb":-0.100,"eps":2.60,"epc":4.20,"geoidSep":46.600,"eph":13.300,"sep":24.130}"#;
        let sky_line = br#"{"class":"SKY","device":"/dev/ttyAMA0","gdop":1.79,"hdop":0.70,"pdop":1.27,"tdop":0.81,"xdop":0.54,"ydop":0.68,"vdop":1.06,"nSat":10,"uSat":9,"satellites":[{"PRN":6,"gnssid":0,"svid":6,"az":40.0,"el":19.0,"ss":31.0,"used":true}]}"#;
        let lines = vec![tpv_line.as_slice(), sky_line.as_slice()];
        let (tpv_data, sky_data) = GPSSocket::parse_gpsd_data(&lines);

        assert!(tpv_data.is_some());
        assert!(sky_data.is_some());
    }

    #[test]
    fn test_parse_skip_short_lines() {
        let short_line = b"short";
        let valid_line = br#"{"class":"TPV","device":"/dev/ttyAMA0","mode":3,"time":"2026-01-14T12:19:37.000Z","leapseconds":18,"ept":0.005,"lat":50.970930167,"lon":6.936929333,"altHAE":110.2000,"altMSL":63.6000,"alt":63.6000,"epx":0.900,"epy":1.300,"epv":2.100,"magvar":2.1,"speed":0.015,"climb":-0.100,"eps":2.60,"epc":4.20,"geoidSep":46.600,"eph":13.300,"sep":24.130}"#;
        let lines = vec![short_line.as_slice(), valid_line.as_slice()];
        let (tpv_data, sky_data) = GPSSocket::parse_gpsd_data(&lines);

        assert!(tpv_data.is_some());
        assert!(sky_data.is_none());
    }

    #[test]
    fn test_parse_ignore_other_classes() {
        let devices_line = br#"{"class":"DEVICES","devices":[]}"#;
        let pps_line = br#"{"class":"PPS","device":"/dev/ttyAMA0","real_sec":1768393177,"real_nsec":0}"#;
        let lines = vec![devices_line.as_slice(), pps_line.as_slice()];
        let (tpv_data, sky_data) = GPSSocket::parse_gpsd_data(&lines);

        assert!(tpv_data.is_none());
        assert!(sky_data.is_none());
    }
}
