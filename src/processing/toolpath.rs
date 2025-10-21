//! Toolpath Representation - Task 64
//!
//! Represents G-code as motion segments for visualization and analysis.

/// Motion segment type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    /// Rapid positioning
    Rapid,
    /// Linear motion
    Linear,
    /// Clockwise arc
    ArcCW,
    /// Counter-clockwise arc
    ArcCCW,
    /// Dwell
    Dwell,
}

/// A motion segment
#[derive(Debug, Clone)]
pub struct Segment {
    /// Segment type
    pub segment_type: SegmentType,
    /// Start X
    pub start_x: f64,
    /// Start Y
    pub start_y: f64,
    /// Start Z
    pub start_z: f64,
    /// End X
    pub end_x: f64,
    /// End Y
    pub end_y: f64,
    /// End Z
    pub end_z: f64,
    /// Feed rate
    pub feed_rate: Option<f64>,
}

impl Segment {
    /// Calculate segment length
    pub fn length(&self) -> f64 {
        let dx = self.end_x - self.start_x;
        let dy = self.end_y - self.start_y;
        let dz = self.end_z - self.start_z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Estimate execution time
    pub fn estimate_time(&self) -> f64 {
        if let Some(feed) = self.feed_rate {
            if feed > 0.0 {
                return self.length() / feed;
            }
        }
        0.0
    }
}

/// Complete toolpath
#[derive(Debug, Clone, Default)]
pub struct Toolpath {
    /// Motion segments
    pub segments: Vec<Segment>,
}

impl Toolpath {
    /// Create new toolpath
    pub fn new() -> Self {
        Self::default()
    }

    /// Add segment
    pub fn add_segment(&mut self, segment: Segment) {
        self.segments.push(segment);
    }

    /// Total toolpath length
    pub fn total_length(&self) -> f64 {
        self.segments.iter().map(|s| s.length()).sum()
    }

    /// Total execution time
    pub fn total_time(&self) -> f64 {
        self.segments.iter().map(|s| s.estimate_time()).sum()
    }

    /// Get rapid vs cutting distance
    pub fn rapid_vs_cutting(&self) -> (f64, f64) {
        let mut rapid = 0.0;
        let mut cutting = 0.0;

        for seg in &self.segments {
            let len = seg.length();
            if seg.segment_type == SegmentType::Rapid {
                rapid += len;
            } else {
                cutting += len;
            }
        }

        (rapid, cutting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_length() {
        let seg = Segment {
            segment_type: SegmentType::Linear,
            start_x: 0.0,
            start_y: 0.0,
            start_z: 0.0,
            end_x: 3.0,
            end_y: 4.0,
            end_z: 0.0,
            feed_rate: Some(100.0),
        };
        assert_eq!(seg.length(), 5.0);
    }

    #[test]
    fn test_toolpath() {
        let mut path = Toolpath::new();
        let seg = Segment {
            segment_type: SegmentType::Linear,
            start_x: 0.0,
            start_y: 0.0,
            start_z: 0.0,
            end_x: 10.0,
            end_y: 0.0,
            end_z: 0.0,
            feed_rate: Some(100.0),
        };
        path.add_segment(seg);
        assert_eq!(path.total_length(), 10.0);
    }
}
