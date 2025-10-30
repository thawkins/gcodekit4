//! Canvas renderer for designer shapes
//! Renders shapes to an image buffer for display in the UI

use image::{ImageBuffer, Rgb, RgbImage};
use crate::designer::{Canvas, ShapeType};

const BG_COLOR: Rgb<u8> = Rgb([52, 73, 94]); // #34495e
const SHAPE_COLOR: Rgb<u8> = Rgb([52, 152, 219]); // #3498db
const SELECTION_COLOR: Rgb<u8> = Rgb([255, 235, 59]); // #ffeb3b
const HANDLE_SIZE: i32 = 12;

/// Render canvas shapes to an image buffer
pub fn render_canvas(canvas: &Canvas, width: u32, height: u32, _zoom: f32, _pan_x: f32, _pan_y: f32) -> RgbImage {
    let mut img = ImageBuffer::from_pixel(width, height, BG_COLOR);
    
    // Get viewport for coordinate transformations
    let viewport = canvas.viewport();
    
    // Render each shape
    for shape_obj in canvas.shapes() {
        let (x1, y1, x2, y2) = shape_obj.shape.bounding_box();
        
        // Convert world coordinates to screen coordinates using viewport transformation
        let (screen_x1, screen_y1) = viewport.world_to_pixel(x1, y1);
        let (screen_x2, screen_y2) = viewport.world_to_pixel(x2, y2);
        
        let screen_x1 = screen_x1 as i32;
        let screen_y1 = screen_y1 as i32;
        let screen_x2 = screen_x2 as i32;
        let screen_y2 = screen_y2 as i32;
        
        // Render based on shape type
        match shape_obj.shape.shape_type() {
            ShapeType::Rectangle => {
                draw_rectangle(&mut img, screen_x1, screen_y1, screen_x2, screen_y2, SHAPE_COLOR);
                if shape_obj.selected {
                    draw_selection_box(&mut img, screen_x1, screen_y1, screen_x2, screen_y2);
                }
            }
            ShapeType::Circle => {
                // Calculate circle center and radius in world coordinates
                let center_x = (x1 + x2) / 2.0;
                let center_y = (y1 + y2) / 2.0;
                let radius_world = ((x2 - x1) / 2.0).abs();
                
                // Convert center to screen coordinates
                let (cx, cy) = viewport.world_to_pixel(center_x, center_y);
                
                // Calculate screen radius (use viewport zoom scale)
                let radius_screen = (radius_world * viewport.zoom()) as i32;
                
                draw_circle(&mut img, cx as i32, cy as i32, radius_screen, SHAPE_COLOR);
                if shape_obj.selected {
                    let r = radius_screen;
                    draw_selection_box(&mut img, (cx as i32) - r, (cy as i32) - r, (cx as i32) + r, (cy as i32) + r);
                }
            }
            ShapeType::Line => {
                draw_line(&mut img, screen_x1, screen_y1, screen_x2, screen_y2, SHAPE_COLOR);
                if shape_obj.selected {
                    draw_selection_box(&mut img, screen_x1.min(screen_x2), screen_y1.min(screen_y2),
                                     screen_x1.max(screen_x2), screen_y1.max(screen_y2));
                }
            }
        }
    }
    
    img
}

/// Draw a filled rectangle
fn draw_rectangle(img: &mut RgbImage, x1: i32, y1: i32, x2: i32, y2: i32, color: Rgb<u8>) {
    let min_x = x1.min(x2).max(0);
    let max_x = x1.max(x2).min(img.width() as i32 - 1);
    let min_y = y1.min(y2).max(0);
    let max_y = y1.max(y2).min(img.height() as i32 - 1);
    
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                // Draw filled
                if x == min_x || x == max_x || y == min_y || y == max_y {
                    img.put_pixel(x as u32, y as u32, color);
                }
            }
        }
    }
}

/// Draw a circle
fn draw_circle(img: &mut RgbImage, cx: i32, cy: i32, radius: i32, color: Rgb<u8>) {
    for angle in 0..360 {
        let rad = (angle as f32).to_radians();
        let x = (cx as f32 + radius as f32 * rad.cos()) as i32;
        let y = (cy as f32 + radius as f32 * rad.sin()) as i32;
        if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
            img.put_pixel(x as u32, y as u32, color);
        }
    }
}

/// Draw a line using Bresenham's algorithm
fn draw_line(img: &mut RgbImage, x1: i32, y1: i32, x2: i32, y2: i32, color: Rgb<u8>) {
    let mut x = x1;
    let mut y = y1;
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    
    loop {
        if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
            img.put_pixel(x as u32, y as u32, color);
        }
        
        if x == x2 && y == y2 {
            break;
        }
        
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

/// Draw selection box with handles
fn draw_selection_box(img: &mut RgbImage, x1: i32, y1: i32, x2: i32, y2: i32) {
    // Draw selection outline
    draw_rectangle_outline(img, x1, y1, x2, y2, SELECTION_COLOR);
    
    // Draw resize handles at corners and center
    let handles = [
        (x1, y1),                    // Top-left
        (x2, y1),                    // Top-right
        (x1, y2),                    // Bottom-left
        (x2, y2),                    // Bottom-right
        ((x1 + x2) / 2, (y1 + y2) / 2), // Center
    ];
    
    for (hx, hy) in &handles {
        draw_handle(img, *hx, *hy);
    }
}

/// Draw a rectangle outline
fn draw_rectangle_outline(img: &mut RgbImage, x1: i32, y1: i32, x2: i32, y2: i32, color: Rgb<u8>) {
    // Top and bottom
    for x in x1.min(x2)..=x1.max(x2) {
        if x >= 0 && x < img.width() as i32 {
            if y1 >= 0 && y1 < img.height() as i32 {
                img.put_pixel(x as u32, y1 as u32, color);
            }
            if y2 >= 0 && y2 < img.height() as i32 {
                img.put_pixel(x as u32, y2 as u32, color);
            }
        }
    }
    
    // Left and right
    for y in y1.min(y2)..=y1.max(y2) {
        if y >= 0 && y < img.height() as i32 {
            if x1 >= 0 && x1 < img.width() as i32 {
                img.put_pixel(x1 as u32, y as u32, color);
            }
            if x2 >= 0 && x2 < img.width() as i32 {
                img.put_pixel(x2 as u32, y as u32, color);
            }
        }
    }
}

/// Draw a resize handle
fn draw_handle(img: &mut RgbImage, cx: i32, cy: i32) {
    let half = HANDLE_SIZE / 2;
    for dy in -half..=half {
        for dx in -half..=half {
            let x = cx + dx;
            let y = cy + dy;
            if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
                img.put_pixel(x as u32, y as u32, SELECTION_COLOR);
            }
        }
    }
}
