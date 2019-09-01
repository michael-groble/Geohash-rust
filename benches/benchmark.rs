#![feature(test)]
extern crate test;
extern crate geohash;

#[bench]
fn test_iteration(b: &mut test::Bencher) {
    let bounds = geohash::GeohashBits::from_hash("dp3").bounding_box();
    let mut n = 0;
    b.iter(|| {
        let iter = geohash::GeohashIterator::new(bounds, 16);
        n = iter.count();
    });
    assert_eq!(n, 131841)
}
