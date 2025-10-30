//! Viewport and coordinate transformation for canvas rendering.
//!
//! Handles conversion between pixel coordinates (screen space) and world coordinates
//! (design space). Manages zoom and pan operations with proper coordinate mapping.

use super::shapes::Point;

/// Represents the viewport transformation state (zoom and pan).
#[derive(Debug, Clone)]
pub struct Viewport {
    zoom: f64,
    pan_x: f64,
    pan_y: f64,
    canvas_width: f64,
    canvas_height: f64,
}

impl Viewport {
    /// Creates a new viewport with initial dimensions.
    pub fn new(canvas_width: f64, canvas_height: f64) -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            canvas_width,
            canvas_height,
        }
    }

    /// Gets the canvas width.
    pub fn canvas_width(&self) -> f64 {
        self.canvas_width
    }

    /// Gets the canvas height.
    pub fn canvas_height(&self) -> f64 {
        self.canvas_height
    }

    /// Sets the canvas dimensions (typically called when window resizes).
    pub fn set_canvas_size(&mut self, width: f64, height: f64) {
        self.canvas_width = width;
        self.canvas_height = height;
    }

    /// Gets the current zoom level (1.0 = 100%).
    pub fn zoom(&self) -> f64 {
        self.zoom
    }

    /// Sets the zoom level, constrained between 0.1 and 10.0.
    pub fn set_zoom(&mut self, zoom: f64) {
        if zoom > 0.1 && zoom < 10.0 {
            self.zoom = zoom;
        }
    }

    /// Zooms in by multiplying current zoom by 1.2.
    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom * 1.2);
    }

    /// Zooms out by dividing current zoom by 1.2.
    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom / 1.2);
    }

    /// Resets zoom to 1.0 (100%).
    pub fn reset_zoom(&mut self) {
        self.zoom = 1.0;
    }

    /// Gets the pan offset (X coordinate).
    pub fn pan_x(&self) -> f64 {
        self.pan_x
    }

    /// Gets the pan offset (Y coordinate).
    pub fn pan_y(&self) -> f64 {
        self.pan_y
    }

    /// Sets the pan offset.
    pub fn set_pan(&mut self, x: f64, y: f64) {
        self.pan_x = x;
        self.pan_y = y;
    }

    /// Pans by a delta amount.
    pub fn pan_by(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }

    /// Resets pan to origin (0, 0).
    pub fn reset_pan(&mut self) {
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Converts pixel coordinates to world coordinates.
    ///
    /// Pixel coordinates are in screen space (0,0 at top-left).
    /// World coordinates are in design space.
    ///
    /// The transformation accounts for:
    /// - Pan offset (translation)
    /// - Zoom level (scaling)
    ///
    /// Formula:
    /// ```
    /// world_x = (pixel_x - pan_x) / zoom
    /// world_y = (pixel_y - pan_y) / zoom
    /// ```
    pub fn pixel_to_world(&self, pixel_x: f64, pixel_y: f64) -> Point {
        let world_x = (pixel_x - self.pan_x) / self.zoom;
        let world_y = (pixel_y - self.pan_y) / self.zoom;
        Point::new(world_x, world_y)
    }

    /// Converts world coordinates to pixel coordinates.
    ///
    /// Inverse of `pixel_to_world`.
    ///
    /// Formula:
    /// ```
    /// pixel_x = world_x * zoom + pan_x
    /// pixel_y = world_y * zoom + pan_y
    /// ```
    pub fn world_to_pixel(&self, world_x: f64, world_y: f64) -> (f64, f64) {
        let pixel_x = world_x * self.zoom + self.pan_x;
        let pixel_y = world_y * self.zoom + self.pan_y;
        (pixel_x, pixel_y)
    }

    /// Converts world coordinates to pixel coordinates (using Point).
    pub fn world_point_to_pixel(&self, point: &Point) -> (f64, f64) {
        self.world_to_pixel(point.x, point.y)
    }

    /// Fits the given bounding box into the viewport with padding.
    ///
    /// # Arguments
    /// * `min_x`, `min_y` - Top-left corner of bounding box
    /// * `max_x`, `max_y` - Bottom-right corner of bounding box
    /// * `padding` - Percentage of viewport to reserve as padding (0.0 - 1.0)
    ///
    /// Centers the content and calculates appropriate zoom level.
    pub fn fit_to_bounds(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64, padding: f64) {
        if min_x >= max_x || min_y >= max_y {
            return;
        }

        let width = max_x - min_x;
        let height = max_y - min_y;

        // Calculate zoom to fit content in viewport with padding
        let padding_factor = 1.0 - (padding * 2.0);
        let zoom_x = (self.canvas_width * padding_factor) / width;
        let zoom_y = (self.canvas_height * padding_factor) / height;

        // Use the smaller zoom to fit everything
        let new_zoom = zoom_x.min(zoom_y).max(0.1).min(10.0);

        // Center the content
        let content_pixel_width = width * new_zoom;
        let content_pixel_height = height * new_zoom;

        let center_x = self.canvas_width / 2.0 - content_pixel_width / 2.0;
        let center_y = self.canvas_height / 2.0 - content_pixel_height / 2.0;

        self.zoom = new_zoom;
        self.pan_x = center_x - min_x * new_zoom;
        self.pan_y = center_y - min_y * new_zoom;
    }

    /// Fits the viewport to show all content with optional padding.
    /// Equivalent to fit_to_bounds with 10% padding.
    pub fn fit_to_view(&mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        self.fit_to_bounds(min_x, min_y, max_x, max_y, 0.1);
    }

    /// Zooms to a point, maintaining that point's screen position.
    ///
    /// Useful for "zoom to cursor" functionality.
    ///
    /// # Arguments
    /// * `world_point` - The world coordinate to zoom to
    /// * `new_zoom` - The new zoom level
    pub fn zoom_to_point(&mut self, world_point: &Point, new_zoom: f64) {
        if new_zoom <= 0.1 || new_zoom >= 10.0 {
            return;
        }

        // Get pixel position of world point (at current zoom/pan)
        let (pixel_x, pixel_y) = self.world_to_pixel(world_point.x, world_point.y);

        // Calculate new pan to keep pixel position fixed
        self.zoom = new_zoom;
        self.pan_x = pixel_x - world_point.x * new_zoom;
        self.pan_y = pixel_y - world_point.y * new_zoom;
    }

    /// Zooms in at a specific world point (maintaining cursor position).
    pub fn zoom_in_at(&mut self, world_point: &Point) {
        self.zoom_to_point(world_point, self.zoom * 1.2);
    }

    /// Zooms out at a specific world point (maintaining cursor position).
    pub fn zoom_out_at(&mut self, world_point: &Point) {
        self.zoom_to_point(world_point, self.zoom / 1.2);
    }

    /// Centers the viewport on a world coordinate.
    pub fn center_on(&mut self, world_x: f64, world_y: f64) {
        self.pan_x = self.canvas_width / 2.0 - world_x * self.zoom;
        self.pan_y = self.canvas_height / 2.0 - world_y * self.zoom;
    }

    /// Centers the viewport on a point.
    pub fn center_on_point(&mut self, point: &Point) {
        self.center_on(point.x, point.y);
    }

    /// Resets viewport to default state (1:1 zoom, no pan).
    pub fn reset(&mut self) {
        self.zoom = 1.0;
        self.pan_x = 0.0;
        self.pan_y = 0.0;
    }

    /// Gets viewport info as a string (for debugging or display).
    pub fn to_string(&self) -> String {
        format!(
            "Zoom: {:.2}x | Pan: ({:.1}, {:.1})",
            self.zoom, self.pan_x, self.pan_y
        )
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_creation() {
        let vp = Viewport::new(800.0, 600.0);
        assert_eq!(vp.zoom(), 1.0);
        assert_eq!(vp.pan_x(), 0.0);
        assert_eq!(vp.pan_y(), 0.0);
    }

    #[test]
    fn test_pixel_to_world_no_transform() {
        let vp = Viewport::new(800.0, 600.0);
        let world = vp.pixel_to_world(100.0, 200.0);
        assert_eq!(world.x, 100.0);
        assert_eq!(world.y, 200.0);
    }

    #[test]
    fn test_pixel_to_world_with_zoom() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_zoom(2.0);
        let world = vp.pixel_to_world(200.0, 400.0);
        assert_eq!(world.x, 100.0);
        assert_eq!(world.y, 200.0);
    }

    #[test]
    fn test_pixel_to_world_with_pan() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_pan(50.0, 75.0);
        let world = vp.pixel_to_world(150.0, 275.0);
        assert_eq!(world.x, 100.0);
        assert_eq!(world.y, 200.0);
    }

    #[test]
    fn test_pixel_to_world_with_zoom_and_pan() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_zoom(2.0);
        vp.set_pan(100.0, 150.0);
        let world = vp.pixel_to_world(300.0, 550.0);
        assert_eq!(world.x, 100.0);
        assert_eq!(world.y, 200.0);
    }

    #[test]
    fn test_world_to_pixel_no_transform() {
        let vp = Viewport::new(800.0, 600.0);
        let (pixel_x, pixel_y) = vp.world_to_pixel(100.0, 200.0);
        assert_eq!(pixel_x, 100.0);
        assert_eq!(pixel_y, 200.0);
    }

    #[test]
    fn test_world_to_pixel_with_zoom() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_zoom(2.0);
        let (pixel_x, pixel_y) = vp.world_to_pixel(100.0, 200.0);
        assert_eq!(pixel_x, 200.0);
        assert_eq!(pixel_y, 400.0);
    }

    #[test]
    fn test_world_to_pixel_with_pan() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_pan(50.0, 75.0);
        let (pixel_x, pixel_y) = vp.world_to_pixel(100.0, 200.0);
        assert_eq!(pixel_x, 150.0);
        assert_eq!(pixel_y, 275.0);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_zoom(2.5);
        vp.set_pan(75.0, 125.0);

        let original = Point::new(123.45, 456.78);
        let (pixel_x, pixel_y) = vp.world_to_pixel(original.x, original.y);
        let roundtrip = vp.pixel_to_world(pixel_x, pixel_y);

        assert!((roundtrip.x - original.x).abs() < 0.01);
        assert!((roundtrip.y - original.y).abs() < 0.01);
    }

    #[test]
    fn test_zoom_constraints() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_zoom(0.05); // Too small
        assert!(vp.zoom() > 0.05);

        vp.set_zoom(15.0); // Too large
        assert!(vp.zoom() < 15.0);
    }

    #[test]
    fn test_zoom_in_out() {
        let mut vp = Viewport::new(800.0, 600.0);
        let initial = vp.zoom();
        vp.zoom_in();
        assert!(vp.zoom() > initial);

        vp.zoom_out();
        assert!((vp.zoom() - initial).abs() < 0.01);
    }

    #[test]
    fn test_center_on_point() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_zoom(1.0);
        vp.center_on(100.0, 200.0);

        let world = vp.pixel_to_world(400.0, 300.0);
        assert!((world.x - 100.0).abs() < 0.01);
        assert!((world.y - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_fit_to_bounds() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.fit_to_bounds(0.0, 0.0, 100.0, 100.0, 0.1);

        assert!(vp.zoom() > 1.0); // Should zoom in to fit small content
    }

    #[test]
    fn test_reset() {
        let mut vp = Viewport::new(800.0, 600.0);
        vp.set_zoom(2.5);
        vp.set_pan(100.0, 200.0);
        vp.reset();

        assert_eq!(vp.zoom(), 1.0);
        assert_eq!(vp.pan_x(), 0.0);
        assert_eq!(vp.pan_y(), 0.0);
    }
}
