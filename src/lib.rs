use std::f64;
use std::f64::MAX;

pub struct Location {
    pub longitude: f64,
    pub latitude: f64
}

pub struct BoundingBox {
    min: Location,
    max: Location
}

#[derive(Clone, Copy)]
pub enum Precision {
    Bits(u8),
    Characters(u8)
}

pub struct GeohashBits {
    bits: u64,
    precision: Precision
}

const RADIANS_PER_DEGREE: f64 = std::f64::consts::PI / 180.0;
const EARTH_DISTANCE_METERS: f64 = 2.0 * 6_371_000.0;
type LocationRange = std::ops::RangeInclusive<f64>;
const LONGITUDE_RANGE: LocationRange = (-180.0..=180.0);
const LATITUDE_RANGE: LocationRange = (-90.0..=90.0);

impl Location {
    pub fn validate_range(&self) {
        assert!(LONGITUDE_RANGE.contains(&self.longitude), "longitude out of range");
        assert!(LATITUDE_RANGE.contains(&self.latitude), "latitude out of range");
    }

    pub fn distance_in_meters(&self, to: &Location) -> f64 {
        let self_lat = RADIANS_PER_DEGREE * self.latitude;
        let to_lat   = RADIANS_PER_DEGREE * to.latitude;
        let delta_lat = to_lat - self_lat;
        let delta_lon = RADIANS_PER_DEGREE * (to.longitude - self.longitude);

        let sin_half_lat = (0.5 * delta_lat).sin();
        let sin_half_lon = (0.5 * delta_lon).sin();

        let x = sin_half_lat * sin_half_lat + sin_half_lon * sin_half_lon * self_lat.cos() * to_lat.cos();
        let arc = x.sqrt().asin(); // only good for smallish angles, otherwise user atan2
        EARTH_DISTANCE_METERS * arc
    }
}

impl BoundingBox {
    pub fn enclosing(a: &Location, b: &Location) -> BoundingBox {
        a.validate_range();
        b.validate_range();
        BoundingBox {
            min: Location {
                latitude: a.latitude.min(b.latitude),
                longitude: a.longitude.min(b.longitude)
            },
            max: Location {
                latitude: a.latitude.max(b.latitude),
                longitude: a.longitude.max(b.longitude)
            }
        }
    }

    pub fn min(&self) -> &Location {
        &self.min
    }

    pub fn max(&self) -> &Location {
        &self.max
    }

    pub fn center(&self) -> Location {
        Location {
            longitude: 0.5 * (self.min.longitude + self.max.longitude),
            latitude:  0.5 * (self.min.latitude + self.max.latitude)
        }
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        // TODO account for wraparound at 180th meridian
        if self.max.longitude < other.min.longitude ||
            self.max.latitude < other.min.latitude ||
            self.min.longitude > other.max.longitude ||
            self.min.latitude > other.max.latitude {
            return false
        }
        true
    }
}

const MAX_BINARY_PRECISION: u8 = 32;

fn binary_precision(precision: &Precision) -> u8 {
    match precision {
        &Precision::Bits(n) => n,
        &Precision::Characters(n) => (0.5 * (5 * n) as f32).ceil() as u8
    }
}

fn max_binary_value(binary_precision: u8) -> f64 {
    (1 << binary_precision as u64) as f64
}

fn float_to_bits(value: f64, range: &LocationRange, max_binary_value: f64) -> u32 {
    let fraction = (value - *range.start()) / (range.end() - range.start());
    (fraction * max_binary_value) as u32
}

fn bits_to_float(bits: u32, range: &LocationRange, max_binary_value: f64) -> f64 {
    let fraction = (bits as f64) / max_binary_value;
    *range.start() + fraction * (range.end() - range.start())
}

fn interleave_bits(even_bits: u32, odd_bits: u32) -> u64 {
    (even_bits + odd_bits) as u64
}

impl GeohashBits {

    pub fn from_location(location: &Location, precision: Precision) -> GeohashBits {
        location.validate_range();
        let binary_precision = binary_precision(&precision);
        assert!((1..=MAX_BINARY_PRECISION).contains(&binary_precision), "precision out of range");
        let max_binary_value = max_binary_value(binary_precision);

        let longitude_bits = float_to_bits(location.longitude, &LONGITUDE_RANGE, max_binary_value);
        let latitude_bits  = float_to_bits(location.latitude, &LATITUDE_RANGE, max_binary_value);

        GeohashBits {
            bits: interleave_bits(latitude_bits, longitude_bits),
            precision
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use crate::Location;
    use crate::BoundingBox;

    #[test]
    fn test_distance() {
        let a = Location { longitude: -9.10, latitude: 51.5 };
        let b = Location { longitude: -9.11, latitude: 51.6 };

        assert_approx_eq!(a.distance_in_meters(&b), 11140.9, 0.1);
    }

    #[test]
    #[should_panic]
    fn test_validate_range() {
        Location { longitude: 181.0, latitude: 0.0 }.validate_range();
    }

    fn bbox() -> BoundingBox {
        BoundingBox::enclosing(
            &Location {latitude: 1.0, longitude: 3.0 },
            &Location {latitude: 2.0, longitude: 2.0 }
        )
    }

    #[test]
    fn test_bounding_box_initialization() {
        let bbox = bbox();
        assert_approx_eq!(bbox.min().latitude,  1.0, 1e-5);
        assert_approx_eq!(bbox.min().longitude, 2.0, 1e-5);
        assert_approx_eq!(bbox.max().latitude,  2.0, 1e-5);
        assert_approx_eq!(bbox.max().longitude, 3.0, 1e-5);
    }

    #[test]
    fn test_center() {
        let center = bbox().center();
        assert_approx_eq!(center.latitude,  1.5, 1e-5);
        assert_approx_eq!(center.longitude, 2.5, 1e-5);
    }

    #[test]
    fn test_intersecting() {
        let other = BoundingBox::enclosing(
            &Location {latitude: 1.5, longitude: 2.5 },
            &Location {latitude: 2.5, longitude: 3.5 }
        );
        assert_eq!(other.intersects(&bbox()), true);
    }

    #[test]
    fn test_non_intersecting() {
        let other = BoundingBox::enclosing(
            &Location {latitude: 2.1, longitude: 3.1 },
            &Location {latitude: 3.0, longitude: 4.0 }
        );
        assert_eq!(other.intersects(&bbox()), false);
    }
}
