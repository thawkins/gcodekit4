//! Rendering optimization using viewport culling and spatial indexing
//!
//! Provides efficient rendering by only drawing shapes visible in the viewport.

use crate::spatial_index::{Bounds, SpatialIndex};

/// Render optimizer that uses spatial indexing and viewport culling
pub struct RenderOptimizer {
    spatial_index: SpatialIndex,
    viewport_bounds: Bounds,
    culled_count: usize,
    drawn_count: usize,
    frame_count: usize,
}

impl RenderOptimizer {
    /// Create new render optimizer
    pub fn new(world_bounds: Bounds) -> Self {
        Self {
            spatial_index: SpatialIndex::new(world_bounds, 8, 16),
            viewport_bounds: world_bounds,
            culled_count: 0,
            drawn_count: 0,
            frame_count: 0,
        }
    }

    /// Update viewport bounds for culling
    pub fn update_viewport(&mut self, bounds: Bounds) {
        self.viewport_bounds = bounds;
    }

    /// Add shape to spatial index
    pub fn add_shape(&mut self, id: u64, bounds: &Bounds) {
        self.spatial_index.insert(id, bounds);
    }

    /// Get shapes visible in viewport (culled set)
    pub fn get_visible_shapes(&mut self) -> Vec<u64> {
        let visible = self.spatial_index.query(&self.viewport_bounds);
        self.culled_count = 0;
        self.drawn_count = visible.len();
        visible
    }

    /// Clear all shapes from index
    pub fn clear(&mut self) {
        self.spatial_index.clear();
        self.culled_count = 0;
        self.drawn_count = 0;
    }

    /// Get culling statistics
    pub fn stats(&self) -> RenderStats {
        RenderStats {
            frame_count: self.frame_count,
            shapes_drawn: self.drawn_count,
            shapes_culled: self.culled_count,
            viewport_bounds: self.viewport_bounds,
        }
    }

    /// Mark frame rendered
    pub fn next_frame(&mut self) {
        self.frame_count += 1;
    }
}

impl Default for RenderOptimizer {
    fn default() -> Self {
        Self::new(Bounds::new(-1000.0, -1000.0, 1000.0, 1000.0))
    }
}

/// Statistics for rendering performance
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub frame_count: usize,
    pub shapes_drawn: usize,
    pub shapes_culled: usize,
    pub viewport_bounds: Bounds,
}

impl RenderStats {
    /// Get total shapes processed
    pub fn total_shapes(&self) -> usize {
        self.shapes_drawn + self.shapes_culled
    }

    /// Get culling efficiency percentage
    pub fn culling_efficiency(&self) -> f64 {
        if self.total_shapes() == 0 {
            0.0
        } else {
            (self.shapes_culled as f64 / self.total_shapes() as f64) * 100.0
        }
    }
}


