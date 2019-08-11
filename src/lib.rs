use std::f64;

pub struct Location {
    longitude: f64,
    latitude: f64
}

const RADIANS_PER_DEGREE: f64 = std::f64::consts::PI / 180.0;
const EARTH_DISTANCE_METERS: f64 = 2.0 * 6_371_000.0;

impl Location {
    pub fn distance_in_meters(&self, to: &Location) -> f64 {
        let self_lat = RADIANS_PER_DEGREE * self.latitude;
        let to_lat   = RADIANS_PER_DEGREE * to.latitude;
        let delta_lat = to_lat - self_lat;
        let delta_lon = RADIANS_PER_DEGREE * (to.longitude - self.longitude);

        let sin_half_lat = (0.5 * delta_lat).sin();
        let sin_half_lon = (0.5 * delta_lon).sin();

        let x = sin_half_lat * sin_half_lat + sin_half_lon * sin_half_lon * self_lat.cos() * to_lat.cos();
        let arc = x.sqrt().asin(); // only good for smallish angles, otherwise user atan2
        EARTH_DISTANCE_METERS * arcsd
    }
}

#[cfg(test)]
mod tests {
    use crate::Location;

    #[test]
    fn test_distance() {
        let a = Location { longitude: -9.10, latitude: 51.5 };
        let b = Location { longitude: -9.11, latitude: 51.6 };

        let error = a.distance_in_meters(&b) - 11140.9;
        assert!(error.abs() <  0.1);
    }
}
