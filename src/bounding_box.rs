
use crate::Location;

#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub(crate) min: Location,
    pub(crate) max: Location
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


#[cfg(test)]
mod tests {
    use assert_approx_eq::assert_approx_eq;
    use crate::BoundingBox;
    use super::Location;

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
