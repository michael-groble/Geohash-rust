use std::f64;
use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::*;

#[cfg(feature = "simd")]
mod ops {
    use packed_simd::u64x2;

    const ALTERNATING_1: u64x2 = u64x2::new(0x5555555555555555, 0x5555555555555555);
    const ALTERNATING_2: u64x2 = u64x2::new(0x3333333333333333, 0x3333333333333333);
    const ALTERNATING_4: u64x2 = u64x2::new(0x0F0F0F0F0F0F0F0F, 0x0F0F0F0F0F0F0F0F);
    const ALTERNATING_8: u64x2 = u64x2::new(0x00FF00FF00FF00FF, 0x00FF00FF00FF00FF);
    const ALTERNATING_16: u64x2 = u64x2::new(0x0000FFFF0000FFFF, 0x0000FFFF0000FFFF);
    const ALTERNATING_32: u64x2 = u64x2::new(0x00000000FFFFFFFF, 0x00000000FFFFFFFF);

    pub fn interleave_bits(even_bits: u32, odd_bits: u32) -> u64 {
        let mut bits = u64x2::new(even_bits as u64, odd_bits as u64);

        bits = (bits | (bits << 16)) & ALTERNATING_16;
        bits = (bits | (bits << 8)) & ALTERNATING_8;
        bits = (bits | (bits << 4)) & ALTERNATING_4;
        bits = (bits | (bits << 2)) & ALTERNATING_2;
        bits = (bits | (bits << 1)) & ALTERNATING_1;

        bits.extract(0) | (bits.extract(1) << 1)
    }

    pub fn deinterleave_bits(interleaved: u64) -> (u32, u32) {
        let mut bits = u64x2::new(interleaved, interleaved >> 1) & ALTERNATING_1;

        bits = (bits | (bits >> 1)) & ALTERNATING_2;
        bits = (bits | (bits >> 2)) & ALTERNATING_4;
        bits = (bits | (bits >> 4)) & ALTERNATING_8;
        bits = (bits | (bits >> 8)) & ALTERNATING_16;
        bits = (bits | (bits >> 16)) & ALTERNATING_32;

        (bits.extract(0) as u32, bits.extract(1) as u32)
    }
}

#[cfg(not(feature = "simd"))]
mod ops {
    pub fn interleave_bits(even_bits: u32, odd_bits: u32) -> u64 {
        let mut e = even_bits as u64;
        let mut o = odd_bits as u64;

        e = (e | (e << 16)) & 0x0000FFFF0000FFFF;
        o = (o | (o << 16)) & 0x0000FFFF0000FFFF;

        e = (e | (e <<  8)) & 0x00FF00FF00FF00FF;
        o = (o | (o <<  8)) & 0x00FF00FF00FF00FF;

        e = (e | (e <<  4)) & 0x0F0F0F0F0F0F0F0F;
        o = (o | (o <<  4)) & 0x0F0F0F0F0F0F0F0F;

        e = (e | (e <<  2)) & 0x3333333333333333;
        o = (o | (o <<  2)) & 0x3333333333333333;

        e = (e | (e <<  1)) & 0x5555555555555555;
        o = (o | (o <<  1)) & 0x5555555555555555;

        e | (o << 1)
    }

    pub fn deinterleave_bits(interleaved: u64) -> (u32, u32) {
        let mut e = interleaved        & 0x5555555555555555;
        let mut o = (interleaved >> 1) & 0x5555555555555555;

        e = (e | (e >>  1)) & 0x3333333333333333;
        o = (o | (o >>  1)) & 0x3333333333333333;

        e = (e | (e >>  2)) & 0x0F0F0F0F0F0F0F0F;
        o = (o | (o >>  2)) & 0x0F0F0F0F0F0F0F0F;

        e = (e | (e >>  4)) & 0x00FF00FF00FF00FF;
        o = (o | (o >>  4)) & 0x00FF00FF00FF00FF;

        e = (e | (e >>  8)) & 0x0000FFFF0000FFFF;
        o = (o | (o >>  8)) & 0x0000FFFF0000FFFF;

        e = (e | (e >> 16)) & 0x00000000FFFFFFFF;
        o = (o | (o >> 16)) & 0x00000000FFFFFFFF;

        (e as u32, o as u32)
    }
}

#[derive(Clone, Copy)]
pub enum Precision {
    Bits(u8),
    Characters(u8)
}

#[derive(Clone, Copy)]
pub struct GeohashBits {
    bits: u64,
    precision: Precision
}


#[derive(PartialEq)]
enum InterleaveSet {
    Odds,
    Evens
}

impl Precision {
    pub fn binary_precision(&self) -> u8 {
        match self {
            &Precision::Bits(n) => n,
            &Precision::Characters(n) => (0.5 * (5 * n) as f32).ceil() as u8
        }
    }

    pub fn character_precision(&self) -> u8 {
        match self {
            &Precision::Bits(n) => (0.4 * n as f64) as u8,
            &Precision::Characters(n) => n
        }
    }

    pub fn max_binary_value(&self) -> f64 {
        (1 << self.binary_precision() as u64) as f64
    }

    pub fn is_odd_characters(&self) -> bool {
        match self {
            &Precision::Bits(_) => false,
            &Precision::Characters(n) => (n % 2) > 0
        }
    }
}

const MAX_BINARY_PRECISION: u8 = 32;

fn float_to_bits(value: f64, range: &LocationRange, max_binary_value: f64) -> u32 {
    let fraction = (value - *range.start()) / (range.end() - range.start());
    (fraction * max_binary_value) as u32
}

fn bits_to_float(bits: u32, range: &LocationRange, max_binary_value: f64) -> f64 {
    let fraction = (bits as f64) / max_binary_value;
    *range.start() + fraction * (range.end() - range.start())
}

const BASE32_CHARACTERS: &[u8; 32] = b"0123456789bcdefghjkmnpqrstuvwxyz";

lazy_static! {
    static ref BASE32_BITS: HashMap<char, u64> = {
        let mut map = HashMap::new();
        for (i, c) in BASE32_CHARACTERS.iter().enumerate() {
            map.insert(char::from(*c), i as u64);
        }
        map
    };
}

impl InterleaveSet {
    pub fn modify_mask(&self) -> u64 {
        match self {
            InterleaveSet::Evens => 0x5555555555555555,
            InterleaveSet::Odds => 0xaaaaaaaaaaaaaaaa
        }
    }

    pub fn keep_mask(&self) -> u64 {
        match self {
            InterleaveSet::Evens => 0xaaaaaaaaaaaaaaaa,
            InterleaveSet::Odds => 0x5555555555555555
        }
    }
}

impl GeohashBits {

    pub fn from_location(location: &Location, precision: Precision) -> GeohashBits {
        location.validate_range();
        let binary_precision = precision.binary_precision();
        assert!((1..=MAX_BINARY_PRECISION).contains(&binary_precision), "precision out of range");
        let max_binary_value = precision.max_binary_value();

        let longitude_bits = float_to_bits(location.longitude, &LONGITUDE_RANGE, max_binary_value);
        let latitude_bits  = float_to_bits(location.latitude, &LATITUDE_RANGE, max_binary_value);

        GeohashBits {
            bits: ops::interleave_bits(latitude_bits, longitude_bits),
            precision
        }
    }

    pub fn from_hash(hash: &str) -> GeohashBits {
        let total_bit_length = 2 * (0.5 * 5.0 * hash.len() as f64).ceil() as u64;
        let mut bits: u64 = 0;
        for (i, c) in hash.chars().enumerate() {
            bits |= BASE32_BITS[&c] << (total_bit_length - 5 * (i as u64 + 1));
        }
        GeohashBits {
            bits,
            precision: Precision::Characters(hash.len() as u8)
        }
    }

    pub fn hash(&self) -> String {
        let character_precision = self.precision.character_precision();
        let total_binary_precision = 2 * self.precision.binary_precision();
        let mut hash = String::with_capacity(character_precision as usize);
        for i in 1..=character_precision {
            // each character is 5 bits
            let index = (self.bits >> (total_binary_precision - i * 5) as u64) & 0x1f;
            hash.push(char::from(BASE32_CHARACTERS[index as usize]));
        }
        hash
    }

    pub fn bits(&self) -> u64 {
        self.bits
    }

    pub fn bounding_box(&self) -> BoundingBox {
        let (mut lat_bits, lon_bits) = ops::deinterleave_bits(self.bits);
        let mut lat_precision = self.precision;
        if lat_precision.is_odd_characters() {
            lat_bits >>= 1;
            lat_precision = Precision::Bits(lat_precision.binary_precision() - 1);
        }
        BoundingBox {
            min: Location {
                longitude: bits_to_float(lon_bits, &LONGITUDE_RANGE, self.precision.max_binary_value()),
                latitude:  bits_to_float(lat_bits, &LATITUDE_RANGE, lat_precision.max_binary_value())
            },
            max: Location {
                longitude: bits_to_float(lon_bits + 1, &LONGITUDE_RANGE, self.precision.max_binary_value()),
                latitude:  bits_to_float(lat_bits + 1, &LATITUDE_RANGE, lat_precision.max_binary_value())
            },
        }
    }

    pub fn neighbor(&self, neighbor: &Neighbor) -> GeohashBits {
        match neighbor {
            Neighbor::North => self.incremented(InterleaveSet::Evens,  1),
            Neighbor::South => self.incremented(InterleaveSet::Evens, -1),
            Neighbor::East =>  self.incremented(InterleaveSet::Odds,   1),
            Neighbor::West =>  self.incremented(InterleaveSet::Odds,  -1)
        }
    }

    fn incremented(&self, set: InterleaveSet, direction: i32) -> GeohashBits {
        if direction == 0 {
            return GeohashBits {
                bits: self.bits,
                precision: self.precision
            }
        }
        let mut modify_bits = self.bits & set.modify_mask();
        let keep_bits = self.bits & set.keep_mask();
        let binary_precision = self.precision.binary_precision() as u64;
        let increment = set.keep_mask() >> (64 - 2 * binary_precision);
        let shift_bits = InterleaveSet::Evens == set && self.precision.is_odd_characters();

        if shift_bits {
            modify_bits >>= 2;
        }

        if direction > 0 {
            modify_bits += increment + 1;
        }
        else {
            modify_bits |= increment;
            modify_bits -= increment + 1;
        }

        if shift_bits {
            modify_bits <<= 2;
        }

        modify_bits &= set.modify_mask() >> (64 - 2 * binary_precision);

        GeohashBits {
            bits: modify_bits | keep_bits,
            precision: self.precision
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use crate::Location;
    use crate::Precision;
    use crate::GeohashBits;

    #[test]
    fn test_even_string_encoding() {
        let bits = GeohashBits::from_location(&Location {longitude: -0.1, latitude: 51.5}, Precision::Characters(12));
        assert_eq!(bits.hash(), "gcpuvxr1jzfd");
    }

    #[test]
    fn test_odd_string_encoding() {
        let bits = GeohashBits::from_location(&Location {longitude: -0.1, latitude: 51.5}, Precision::Characters(11));
        assert_eq!(bits.hash(), "gcpuvxr1jzf");
    }

    #[test]
    #[should_panic]
    fn test_encoding_too_long() {
        let _ = GeohashBits::from_location(&Location {longitude: -0.1, latitude: 51.5}, Precision::Characters(13));
    }

    #[test]
    #[should_panic]
    fn test_invalid_angle() {
        let _ = GeohashBits::from_location(&Location {longitude: -200.0, latitude: 51.5}, Precision::Characters(11));
    }

    #[test]
    fn test_even_string_decoding() {
        let bits = GeohashBits::from_hash("u10hfr2c4pv6");
        assert_eq!(bits.bits(), 0xd041075c4b25766);
        assert_approx_eq!(bits.bounding_box().center().longitude, 0.0999999605119228, 1.0e-13);
        assert_approx_eq!(bits.bounding_box().center().latitude, 51.500000031665,     1.0e-13);
    }

    #[test]
    fn test_odd_string_decoding() {
        let bits = GeohashBits::from_hash("u10hfr2c4pv");
        assert_eq!(bits.bits(), 0xd041075c4b2576);
        assert_approx_eq!(bits.bounding_box().center().longitude, 0.100000128149986, 1.0e-13);
        assert_approx_eq!(bits.bounding_box().center().latitude, 51.5000002831221,   1.0e-13);
    }

    #[test]
    fn test_even_binary_encoding() {
        // match redis precision for comparison
        let bits = GeohashBits::from_location(&Location {longitude: -0.1, latitude: 51.5}, Precision::Bits(26));
        // note redis always returns 11 character hashes "gcpuvxr1jz0",
        // but we would need 55 bits for 11 characters and we only have 52 so we truncate at 10 characters
        assert_eq!(bits.hash(), "gcpuvxr1jz");
        assert_approx_eq!(bits.bounding_box().center().longitude, -0.10000079870223999, 1.0e-13);
        assert_approx_eq!(bits.bounding_box().center().latitude,  51.4999996125698,     1.0e-13);
    }

    #[test]
    fn test_odd_binary_encoding() {
        let bits = GeohashBits::from_location(&Location {longitude: -0.1, latitude: 51.5}, Precision::Bits(25));
        assert_eq!(bits.hash(), "gcpuvxr1jz");
        assert_approx_eq!(bits.bounding_box().center().longitude, -0.0999981164932251, 1.0e-13);
        assert_approx_eq!(bits.bounding_box().center().latitude,  51.4999982714653,    1.0e-13);
    }
}
