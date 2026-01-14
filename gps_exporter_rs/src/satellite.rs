/// Calculate approximate satellite ground position from observer position and satellite azimuth/elevation
///
/// # Arguments
/// * `observer_lat` - Observer latitude in degrees
/// * `observer_lon` - Observer longitude in degrees
/// * `azimuth` - Satellite azimuth in degrees (0=North, 90=East)
/// * `elevation` - Satellite elevation in degrees above horizon
///
/// # Returns
/// * `(satellite_lat, satellite_lon)` - Satellite position in degrees
pub fn calculate_satellite_position(
    observer_lat: f64,
    observer_lon: f64,
    azimuth: f64,
    elevation: f64,
) -> (f64, f64) {
    const SATELLITE_ALTITUDE_KM: f64 = 20200.0;
    const EARTH_RADIUS_KM: f64 = 6371.0;

    let obs_lat_rad = observer_lat.to_radians();
    let obs_lon_rad = observer_lon.to_radians();
    let azimuth_rad = azimuth.to_radians();
    let elevation_rad = elevation.to_radians();

    let zenith_angle = std::f64::consts::PI / 2.0 - elevation_rad;

    // Slant range from observer to satellite
    let slant_range = ((EARTH_RADIUS_KM + SATELLITE_ALTITUDE_KM).powi(2)
        - EARTH_RADIUS_KM.powi(2) * zenith_angle.sin().powi(2))
    .sqrt()
        - EARTH_RADIUS_KM * zenith_angle.cos();

    // Angular distance
    let angular_distance = if slant_range > 0.0 {
        (slant_range * zenith_angle.sin() / (EARTH_RADIUS_KM + SATELLITE_ALTITUDE_KM)).asin()
    } else {
        0.0
    };

    // Calculate new position using great circle calculation
    let sat_lat = (obs_lat_rad.sin() * angular_distance.cos()
        + obs_lat_rad.cos() * angular_distance.sin() * azimuth_rad.cos())
    .asin();

    let sat_lon = obs_lon_rad
        + (azimuth_rad.sin() * angular_distance.sin() * obs_lat_rad.cos())
            .atan2(angular_distance.cos() - obs_lat_rad.sin() * sat_lat.sin());

    let sat_lat_deg = sat_lat.to_degrees();
    let mut sat_lon_deg = sat_lon.to_degrees();

    // Normalize longitude to -180 to 180
    sat_lon_deg = ((sat_lon_deg + 180.0) % 360.0) - 180.0;

    (sat_lat_deg, sat_lon_deg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_satellite_position_calculation() {
        // Observer at: lat=50.970930167, lon=6.936929333
        // Satellite PRN 25: az=319.0, el=84.0 (nearly overhead)
        let observer_lat = 50.970930167;
        let observer_lon = 6.936929333;
        let azimuth = 319.0;
        let elevation = 84.0;

        let (sat_lat, sat_lon) = calculate_satellite_position(observer_lat, observer_lon, azimuth, elevation);

        // At 84° elevation, satellite should be relatively close to observer position
        // but still offset due to 20,200 km altitude
        assert!(sat_lat.is_finite());
        assert!(sat_lon.is_finite());
        assert!(sat_lat >= -90.0 && sat_lat <= 90.0);
        assert!(sat_lon >= -180.0 && sat_lon <= 180.0);
        // Should be roughly northwest of observer (azimuth 319°)
        assert!((sat_lat - observer_lat).abs() < 10.0, "Latitude should be within 10 degrees");
    }

    #[test]
    fn test_satellite_position_low_elevation() {
        // Observer at: lat=50.970930167, lon=6.936929333
        // Satellite PRN 6: az=40.0, el=19.0 (low on horizon)
        let observer_lat = 50.970930167;
        let observer_lon = 6.936929333;
        let azimuth = 40.0;
        let elevation = 19.0;

        let (sat_lat, sat_lon) = calculate_satellite_position(observer_lat, observer_lon, azimuth, elevation);

        // At low elevation, satellite should be farther away
        assert!(sat_lat.is_finite());
        assert!(sat_lon.is_finite());
        assert!(sat_lat >= -90.0 && sat_lat <= 90.0);
        assert!(sat_lon >= -180.0 && sat_lon <= 180.0);
    }

    #[test]
    fn test_satellite_position_south() {
        // Test satellite to the south (azimuth=200.0)
        let observer_lat = 50.970930167;
        let observer_lon = 6.936929333;
        let azimuth = 200.0;
        let elevation = 48.0;

        let (sat_lat, sat_lon) = calculate_satellite_position(observer_lat, observer_lon, azimuth, elevation);

        // South direction should result in lower latitude
        assert!(sat_lat < observer_lat, "Satellite to the south should have lower latitude");
        assert!(sat_lat >= -90.0 && sat_lat <= 90.0);
        assert!(sat_lon >= -180.0 && sat_lon <= 180.0);
    }

    #[test]
    fn test_satellite_position_east() {
        // Test satellite to the east (azimuth=86.0)
        let observer_lat = 50.970930167;
        let observer_lon = 6.936929333;
        let azimuth = 86.0;
        let elevation = 32.0;

        let (sat_lat, sat_lon) = calculate_satellite_position(observer_lat, observer_lon, azimuth, elevation);

        // East direction should result in higher longitude
        assert!(sat_lon > observer_lon, "Satellite to the east should have higher longitude");
        assert!(sat_lat >= -90.0 && sat_lat <= 90.0);
        assert!(sat_lon >= -180.0 && sat_lon <= 180.0);
    }

    #[test]
    fn test_longitude_normalization() {
        // Test that longitude is normalized to -180 to 180
        let observer_lat = 0.0;
        let observer_lon = 179.0;
        let azimuth = 90.0; // East
        let elevation = 45.0;

        let (_, sat_lon) = calculate_satellite_position(observer_lat, observer_lon, azimuth, elevation);

        assert!(sat_lon >= -180.0 && sat_lon <= 180.0, "Longitude must be normalized to -180..180");
    }
}
