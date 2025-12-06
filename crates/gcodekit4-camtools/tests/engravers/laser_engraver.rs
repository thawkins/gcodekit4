use gcodekit4_camtools::laser_engraver::{EngravingParameters, ImageTransformations, RotationAngle, HalftoneMethod};

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

