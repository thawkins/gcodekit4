//! Integration tests for DXF import and parsing (Phase 4.5)

use gcodekit4::designer::{
    DxfArc, DxfCircle, DxfEntity, DxfEntityType, DxfFile, DxfHeader, DxfLine, DxfParser,
    DxfPolyline, DxfText, DxfUnit, Point,
};

#[test]
fn test_dxf_unit_all_conversions() {
    let mm_factor = DxfUnit::Millimeters.to_mm_factor();
    assert_eq!(mm_factor, 1.0);

    let in_factor = DxfUnit::Inches.to_mm_factor();
    assert!((in_factor - 25.4).abs() < 0.01);

    let cm_factor = DxfUnit::Centimeters.to_mm_factor();
    assert_eq!(cm_factor, 10.0);

    let m_factor = DxfUnit::Meters.to_mm_factor();
    assert_eq!(m_factor, 1000.0);
}

#[test]
fn test_dxf_line_endpoints() {
    let line = DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(100.0, 50.0),
        layer: "Geometry".to_string(),
        color: 1,
    };

    let distance = line.start.distance_to(&line.end);
    assert!((distance - 111.8).abs() < 0.1);
}

#[test]
fn test_dxf_circle_properties() {
    let circle = DxfCircle {
        center: Point::new(10.0, 20.0),
        radius: 15.0,
        layer: "Circles".to_string(),
        color: 2,
    };

    let circumference = 2.0 * std::f64::consts::PI * circle.radius;
    assert!((circumference - 94.25).abs() < 0.1);
}

#[test]
fn test_dxf_arc_angle_range() {
    let arc = DxfArc {
        center: Point::new(0.0, 0.0),
        radius: 10.0,
        start_angle: 45.0,
        end_angle: 135.0,
        layer: "Arcs".to_string(),
        color: 3,
    };

    let angle_span = arc.end_angle - arc.start_angle;
    assert_eq!(angle_span, 90.0);
}

#[test]
fn test_dxf_arc_full_circle() {
    let arc = DxfArc {
        center: Point::new(0.0, 0.0),
        radius: 5.0,
        start_angle: 0.0,
        end_angle: 360.0,
        layer: "Circles".to_string(),
        color: 1,
    };

    let angle_span = arc.end_angle - arc.start_angle;
    assert_eq!(angle_span, 360.0);
}

#[test]
fn test_dxf_polyline_open() {
    let polyline = DxfPolyline {
        vertices: vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 10.0),
            Point::new(0.0, 10.0),
        ],
        closed: false,
        layer: "Paths".to_string(),
        color: 1,
    };

    assert_eq!(polyline.vertices.len(), 4);
    assert!(!polyline.closed);
}

#[test]
fn test_dxf_polyline_closed() {
    let mut polyline = DxfPolyline {
        vertices: vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 10.0),
        ],
        closed: false,
        layer: "Polygons".to_string(),
        color: 1,
    };

    polyline.closed = true;
    assert!(polyline.closed);
    assert_eq!(polyline.vertices.len(), 3);
}

#[test]
fn test_dxf_polyline_bounds() {
    let polyline = DxfPolyline {
        vertices: vec![
            Point::new(0.0, 0.0),
            Point::new(20.0, 15.0),
            Point::new(10.0, 30.0),
        ],
        closed: false,
        layer: "0".to_string(),
        color: 256,
    };

    let min_x = polyline
        .vertices
        .iter()
        .map(|p| p.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = polyline
        .vertices
        .iter()
        .map(|p| p.x)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = polyline
        .vertices
        .iter()
        .map(|p| p.y)
        .fold(f64::INFINITY, f64::min);
    let max_y = polyline
        .vertices
        .iter()
        .map(|p| p.y)
        .fold(f64::NEG_INFINITY, f64::max);

    assert_eq!(min_x, 0.0);
    assert_eq!(max_x, 20.0);
    assert_eq!(min_y, 0.0);
    assert_eq!(max_y, 30.0);
}

#[test]
fn test_dxf_text_styling() {
    let text = DxfText {
        content: "Title".to_string(),
        position: Point::new(50.0, 50.0),
        height: 5.0,
        rotation: 45.0,
        layer: "Text".to_string(),
        color: 1,
    };

    assert_eq!(text.content, "Title");
    assert_eq!(text.height, 5.0);
    assert_eq!(text.rotation, 45.0);
}

#[test]
fn test_dxf_entity_layer_access() {
    let line = DxfEntity::Line(DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(1.0, 1.0),
        layer: "MyLayer".to_string(),
        color: 1,
    });

    assert_eq!(line.layer(), "MyLayer");
    assert_eq!(line.entity_type(), DxfEntityType::Line);
}

#[test]
fn test_dxf_entity_color() {
    let circle = DxfEntity::Circle(DxfCircle {
        center: Point::new(0.0, 0.0),
        radius: 1.0,
        layer: "0".to_string(),
        color: 5,
    });

    assert_eq!(circle.color(), 5);
}

#[test]
fn test_dxf_file_add_entities() {
    let mut file = DxfFile::new();

    for i in 0..5 {
        file.add_entity(DxfEntity::Line(DxfLine {
            start: Point::new(0.0, 0.0),
            end: Point::new(i as f64, i as f64),
            layer: "Lines".to_string(),
            color: 1,
        }));
    }

    assert_eq!(file.entity_count(), 5);
}

#[test]
fn test_dxf_file_layer_organization() {
    let mut file = DxfFile::new();

    file.add_entity(DxfEntity::Line(DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(1.0, 1.0),
        layer: "Layer1".to_string(),
        color: 1,
    }));

    file.add_entity(DxfEntity::Line(DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(2.0, 2.0),
        layer: "Layer1".to_string(),
        color: 1,
    }));

    file.add_entity(DxfEntity::Circle(DxfCircle {
        center: Point::new(0.0, 0.0),
        radius: 1.0,
        layer: "Layer2".to_string(),
        color: 1,
    }));

    let layer1 = file.get_layer_entities("Layer1");
    assert!(layer1.is_some());
    assert_eq!(layer1.unwrap().len(), 2);

    let layer2 = file.get_layer_entities("Layer2");
    assert!(layer2.is_some());
    assert_eq!(layer2.unwrap().len(), 1);
}

#[test]
fn test_dxf_file_scale_linear() {
    let mut file = DxfFile::new();

    file.add_entity(DxfEntity::Line(DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(10.0, 10.0),
        layer: "0".to_string(),
        color: 256,
    }));

    file.scale(0.5);

    if let DxfEntity::Line(line) = &file.entities[0] {
        assert_eq!(line.end.x, 5.0);
        assert_eq!(line.end.y, 5.0);
    }
}

#[test]
fn test_dxf_file_scale_circle() {
    let mut file = DxfFile::new();

    file.add_entity(DxfEntity::Circle(DxfCircle {
        center: Point::new(0.0, 0.0),
        radius: 10.0,
        layer: "0".to_string(),
        color: 256,
    }));

    file.scale(2.0);

    if let DxfEntity::Circle(circle) = &file.entities[0] {
        assert_eq!(circle.radius, 20.0);
    }
}

#[test]
fn test_dxf_file_unit_conversion_inches_to_mm() {
    let mut file = DxfFile::new();

    file.add_entity(DxfEntity::Line(DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(1.0, 1.0),
        layer: "0".to_string(),
        color: 256,
    }));

    file.convert_units(DxfUnit::Inches, DxfUnit::Millimeters);

    if let DxfEntity::Line(line) = &file.entities[0] {
        assert!((line.end.x - 25.4).abs() < 0.1);
    }
}

#[test]
fn test_dxf_file_bounding_box() {
    let file = DxfFile::new();
    let (min, max) = file.bounding_box();

    assert_eq!(min, file.header.extents_min);
    assert_eq!(max, file.header.extents_max);
}

#[test]
fn test_dxf_header_version() {
    let header = DxfHeader::default();
    assert_eq!(header.version, "AC1021");
}

#[test]
fn test_dxf_parser_validate_valid() {
    let valid_dxf = "0\nSECTION\n0\nENDSEC";
    let result = DxfParser::validate_header(valid_dxf);
    assert!(result.is_ok());
}

#[test]
fn test_dxf_parser_validate_missing_section() {
    let invalid_dxf = "0\nLINE\n0\nENDSEC";
    let result = DxfParser::validate_header(invalid_dxf);
    assert!(result.is_err());
}

#[test]
fn test_dxf_parser_validate_missing_endsec() {
    let invalid_dxf = "0\nSECTION\n0\nLINE";
    let result = DxfParser::validate_header(invalid_dxf);
    assert!(result.is_err());
}

#[test]
fn test_dxf_parser_parse_simple() {
    let dxf_content = "0\nSECTION\n2\nENTITIES\n0\nLINE\n0\nENDSEC";
    let result = DxfParser::parse(dxf_content);
    assert!(result.is_ok());
}

#[test]
fn test_dxf_file_multiple_layers() {
    let mut file = DxfFile::new();

    for layer_num in 0..3 {
        file.add_entity(DxfEntity::Line(DxfLine {
            start: Point::new(0.0, 0.0),
            end: Point::new(1.0, 1.0),
            layer: format!("Layer{}", layer_num),
            color: 1,
        }));
    }

    let layers = file.layer_names();
    assert_eq!(layers.len(), 3);
}

#[test]
fn test_dxf_mixed_entity_types() {
    let mut file = DxfFile::new();

    file.add_entity(DxfEntity::Line(DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(1.0, 1.0),
        layer: "0".to_string(),
        color: 1,
    }));

    file.add_entity(DxfEntity::Circle(DxfCircle {
        center: Point::new(0.0, 0.0),
        radius: 1.0,
        layer: "0".to_string(),
        color: 1,
    }));

    file.add_entity(DxfEntity::Arc(DxfArc {
        center: Point::new(0.0, 0.0),
        radius: 1.0,
        start_angle: 0.0,
        end_angle: 90.0,
        layer: "0".to_string(),
        color: 1,
    }));

    file.add_entity(DxfEntity::Polyline(DxfPolyline {
        vertices: vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)],
        closed: false,
        layer: "0".to_string(),
        color: 1,
    }));

    file.add_entity(DxfEntity::Text(DxfText {
        content: "Test".to_string(),
        position: Point::new(0.0, 0.0),
        height: 1.0,
        rotation: 0.0,
        layer: "0".to_string(),
        color: 1,
    }));

    assert_eq!(file.entity_count(), 5);
}

#[test]
fn test_dxf_scale_preserves_ratios() {
    let mut file = DxfFile::new();

    file.add_entity(DxfEntity::Line(DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(4.0, 3.0),
        layer: "0".to_string(),
        color: 256,
    }));

    file.scale(10.0);

    if let DxfEntity::Line(line) = &file.entities[0] {
        let new_distance = line.start.distance_to(&line.end);
        assert!((new_distance - 50.0).abs() < 0.1);
    }
}

#[test]
fn test_dxf_file_layer_retrieval() {
    let mut file = DxfFile::new();

    file.add_entity(DxfEntity::Line(DxfLine {
        start: Point::new(0.0, 0.0),
        end: Point::new(1.0, 1.0),
        layer: "TestLayer".to_string(),
        color: 1,
    }));

    let layer = file.get_layer_entities("TestLayer");
    assert!(layer.is_some());
    assert_eq!(layer.unwrap().len(), 1);

    let missing = file.get_layer_entities("NonExistent");
    assert!(missing.is_none());
}
