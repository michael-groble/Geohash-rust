use crate::*;

pub struct GeohashIterator {
    bounds: BoundingBox,
    lat_baseline: GeohashBits,
    current: Option<GeohashBits>,
}

impl GeohashIterator {
    pub fn new(bounds: BoundingBox, bit_precision: u8) -> GeohashIterator {
        let lat_baseline =
            GeohashBits::from_location(&bounds.min(), Precision::Bits(bit_precision));
        GeohashIterator {
            bounds,
            lat_baseline,
            current: Some(lat_baseline),
        }
    }

    fn advance_current(&mut self) {
        // advance eastward until we are out of the bounds then advance northward
        if let Some(bits) = self.current {
            let bits = bits.neighbor(&Neighbor::East);
            if self.bounds.intersects(&bits.bounding_box()) {
                self.current = Some(bits);
            } else {
                self.lat_baseline = self.lat_baseline.neighbor(&Neighbor::North);
                if self.bounds.intersects(&self.lat_baseline.bounding_box()) {
                    self.current = Some(self.lat_baseline);
                } else {
                    self.current = Option::None;
                }
            }
        }
    }
}

impl std::iter::Iterator for GeohashIterator {
    type Item = GeohashBits;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.current.clone();
        self.advance_current();
        value
    }
}

#[cfg(test)]
mod tests {
    use crate::BoundingBox;
    use crate::GeohashIterator;
    use crate::Location;

    #[test]
    fn test_iterator() {
        let bounds = BoundingBox::enclosing(vec![
            Location {
                longitude: 0.09991,
                latitude: 51.49996,
            },
            Location {
                longitude: 0.10059,
                latitude: 51.50028,
            },
        ])
        .unwrap();
        let mut iterator = GeohashIterator::new(bounds, 20);
        assert_eq!(iterator.next().unwrap().hash(), "u10hfr2c");
        assert_eq!(iterator.next().unwrap().hash(), "u10hfr31");
        assert_eq!(iterator.next().unwrap().hash(), "u10hfr2f");
        assert_eq!(iterator.next().unwrap().hash(), "u10hfr34");
        assert!(iterator.next().is_none());
    }
}
