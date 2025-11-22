use gcodekit4_camtools::laser_engraver::{BitmapImageEngraver, EngravingParameters, ImageTransformations, RotationAngle, HalftoneMethod};
use pepecore::svec::{SVec, Shape};
use pepecore::enums::ImgData;

#[test]
fn test_default_parameters() {
    let params = EngravingParameters::default();
    assert_eq!(params.width_mm, 100.0);
    assert_eq!(params.feed_rate, 1000.0);
    assert!(params.bidirectional);
}

#[test]
fn test_transformations_default() {
    let trans = ImageTransformations::default();
    assert!(!trans.mirror_x);
    assert!(!trans.mirror_y);
    assert_eq!(trans.rotation, RotationAngle::Degrees0);
    assert_eq!(trans.halftone, HalftoneMethod::None);
}

#[test]
fn test_mirror_x_svec() {
    let data = vec![1, 2, 3, 4];
    let shape = Shape::new(2, 2, None);
    let mut svec = SVec::new(shape, ImgData::U8(data));
    BitmapImageEngraver::mirror_x_svec(&mut svec).unwrap();
    // First row [1,2] -> [2,1], second row [3,4] -> [4,3]
    match &svec.data {
        ImgData::U8(data) => assert_eq!(data, &vec![2, 1, 4, 3]),
        _ => panic!("Expected U8 data"),
    }
}

#[test]
fn test_rotation_90_degrees_svec() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let shape = Shape::new(3, 2, None);
    let svec = SVec::new(shape, ImgData::U8(data));
    let rotated = BitmapImageEngraver::apply_rotation_svec(svec, RotationAngle::Degrees90).unwrap();
    let (h, w, _) = rotated.shape();
    assert_eq!(w, 3);
    assert_eq!(h, 2);
    match &rotated.data {
        ImgData::U8(data) => {
            assert_eq!(data[0], 5);
            assert_eq!(data[1], 3);
            assert_eq!(data[2], 1);
        }
        _ => panic!("Expected U8 data"),
    }
}
