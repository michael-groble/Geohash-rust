#![feature(test)]
extern crate test;
extern crate geohash;

#[bench]
fn test_iteration(b: &mut test::Bencher) {
    let parent = geohash::GeohashBits::from_hash("dp3");
    b.iter(|| {
        let iter = geohash::GeohashIterator::new(parent.bounding_box(), 16);
        iter.count()
    });
}
