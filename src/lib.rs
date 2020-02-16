use std::f64;

pub mod location;
pub use self::location::Location;

pub mod bounding_box;
pub use self::bounding_box::BoundingBox;

pub mod geohash_bits;
pub use self::geohash_bits::{GeohashBits, Precision};

pub mod geohash_iterator;
pub use self::geohash_iterator::GeohashIterator;

type LocationRange = std::ops::RangeInclusive<f64>;
const LONGITUDE_RANGE: LocationRange = (-180.0..=180.0);
const LATITUDE_RANGE: LocationRange = (-90.0..=90.0);

pub enum Neighbor {
    West,
    East,
    South,
    North,
}
