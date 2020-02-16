#[derive(Clone, Copy)]
pub struct Location {
    pub longitude: f64,
    pub latitude: f64,
}

const RADIANS_PER_DEGREE: f64 = std::f64::consts::PI / 180.0;
const EARTH_DISTANCE_METERS: f64 = 2.0 * 6_371_000.0;

impl Location {
    pub fn validate_range(&self) {
        assert!(
            crate::LONGITUDE_RANGE.contains(&self.longitude),
            "longitude out of range"
        );
        assert!(
            crate::LATITUDE_RANGE.contains(&self.latitude),
            "latitude out of range"
        );
    }

    pub fn distance_in_meters(&self, to: &Location) -> f64 {
        let self_lat = RADIANS_PER_DEGREE * self.latitude;
        let to_lat = RADIANS_PER_DEGREE * to.latitude;
        let delta_lat = to_lat - self_lat;
        let delta_lon = RADIANS_PER_DEGREE * (to.longitude - self.longitude);

        let sin_half_lat = (0.5 * delta_lat).sin();
        let sin_half_lon = (0.5 * delta_lon).sin();

        let x = sin_half_lat * sin_half_lat
            + sin_half_lon * sin_half_lon * self_lat.cos() * to_lat.cos();
        let arc = x.sqrt().asin(); // only good for smallish angles, otherwise user atan2
        EARTH_DISTANCE_METERS * arc
    }
}

#[cfg(test)]
mod tests {
    use crate::Location;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_distance() {
        let a = Location {
            longitude: -9.10,
            latitude: 51.5,
        };
        let b = Location {
            longitude: -9.11,
            latitude: 51.6,
        };

        assert_approx_eq!(a.distance_in_meters(&b), 11140.9, 0.1);
    }

    #[test]
    #[should_panic]
    fn test_validate_range() {
        Location {
            longitude: 181.0,
            latitude: 0.0,
        }
        .validate_range();
    }
}
