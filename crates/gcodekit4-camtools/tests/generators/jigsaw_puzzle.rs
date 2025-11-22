use gcodekit4_camtools::jigsaw_puzzle::{JigsawPuzzleMaker, PuzzleParameters};

#[test]
fn test_default_parameters() {
    let params = PuzzleParameters::default();
    assert_eq!(params.width, 200.0);
    assert_eq!(params.height, 150.0);
    assert_eq!(params.pieces_across, 4);
    assert_eq!(params.pieces_down, 3);
}

#[test]
fn test_parameter_validation() {
    let mut params = PuzzleParameters::default();
    params.width = 10.0;

    let result = JigsawPuzzleMaker::new(params);
    assert!(result.is_err());
}

#[test]
fn test_generate_simple_puzzle() {
    let params = PuzzleParameters::default();
    let mut maker = JigsawPuzzleMaker::new(params).unwrap();

    let result = maker.generate();
    assert!(result.is_ok());
    // We can't access private paths field, but we can check if generation succeeded
    // and maybe check gcode output
}

#[test]
fn test_gcode_generation() {
    let params = PuzzleParameters::default();
    let mut maker = JigsawPuzzleMaker::new(params).unwrap();
    maker.generate().unwrap();

    let gcode = maker.to_gcode(300.0, 3.0);
    assert!(gcode.contains("G21"));
    assert!(gcode.contains("G90"));
    assert!(gcode.contains("M2"));
    assert!(gcode.contains("Jigsaw Puzzle"));
}
