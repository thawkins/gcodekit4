//! Spatial indexing for efficient shape lookup and rendering optimization
//!
//! Provides quadtree-based spatial partitioning for fast shape queries,
//! culling, and collision detection in large designs.

/// Bounds of a region
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Bounds {
    /// Create new bounds
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        Self {
            min_x: min_x.min(max_x),
            min_y: min_y.min(max_y),
            max_x: min_x.max(max_x),
            max_y: min_y.max(max_y),
        }
    }

    /// Get center point
    pub fn center(&self) -> (f64, f64) {
        (
            (self.min_x + self.max_x) / 2.0,
            (self.min_y + self.max_y) / 2.0,
        )
    }

    /// Get width
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Get height
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Check if bounds contains point
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y
    }

    /// Check if bounds intersects another bounds
    pub fn intersects(&self, other: &Bounds) -> bool {
        !(self.max_x < other.min_x
            || self.min_x > other.max_x
            || self.max_y < other.min_y
            || self.min_y > other.max_y)
    }

    /// Check if bounds fully contains another bounds
    pub fn contains_bounds(&self, other: &Bounds) -> bool {
        self.min_x <= other.min_x
            && self.max_x >= other.max_x
            && self.min_y <= other.min_y
            && self.max_y >= other.max_y
    }
}

/// Quadtree node for spatial indexing
#[derive(Debug)]
struct QuadtreeNode {
    bounds: Bounds,
    items: Vec<usize>,
    children: Option<Box<[QuadtreeNode; 4]>>,
    depth: usize,
}

impl QuadtreeNode {
    /// Create new leaf node
    fn new_leaf(bounds: Bounds) -> Self {
        Self {
            bounds,
            items: Vec::new(),
            children: None,
            depth: 0,
        }
    }

    /// Split node into 4 children
    fn split(&mut self) {
        if self.children.is_some() {
            return;
        }

        let (cx, cy) = self.bounds.center();
        let (min_x, min_y, max_x, max_y) =
            (self.bounds.min_x, self.bounds.min_y, self.bounds.max_x, self.bounds.max_y);

        let mut children = Box::new([
            QuadtreeNode::new_leaf(Bounds::new(min_x, min_y, cx, cy)),
            QuadtreeNode::new_leaf(Bounds::new(cx, min_y, max_x, cy)),
            QuadtreeNode::new_leaf(Bounds::new(min_x, cy, cx, max_y)),
            QuadtreeNode::new_leaf(Bounds::new(cx, cy, max_x, max_y)),
        ]);

        for child in children.iter_mut() {
            child.depth = self.depth + 1;
        }

        self.children = Some(children);
    }
}

/// Insert item into node tree (helper function to avoid borrow checker issues)
fn insert_into_node(
    node: &mut QuadtreeNode,
    index: usize,
    bounds: &Bounds,
    max_depth: usize,
    max_items: usize,
) {
    // If bounds don't intersect, skip
    if !node.bounds.intersects(bounds) {
        return;
    }

    // If we have children, try to insert into them
    if node.children.is_some() {
        let children = node.children.as_mut().unwrap();
        for child in children.iter_mut() {
            insert_into_node(child, index, bounds, max_depth, max_items);
        }
        return;
    }

    // Add to current node
    if !node.items.contains(&index) {
        node.items.push(index);
    }

    // Split if necessary
    if node.items.len() > max_items && node.depth < max_depth {
        node.split();

        // Redistribute items among children
        let items: Vec<usize> = node.items.drain(..).collect();
        for item_idx in items {
            let children = node.children.as_mut().unwrap();
            // Insert into only the children that intersect with the item bounds
            for child in children.iter_mut() {
                if child.bounds.intersects(bounds) {
                    if !child.items.contains(&item_idx) {
                        child.items.push(item_idx);
                    }
                }
            }
        }
    }
}

/// Query node tree (helper function)
fn query_node(node: &QuadtreeNode, query_bounds: &Bounds, results: &mut Vec<usize>) {
    if !node.bounds.intersects(query_bounds) {
        return;
    }

    results.extend(&node.items);

    if let Some(children) = &node.children {
        for child in children.iter() {
            query_node(child, query_bounds, results);
        }
    }
}

/// Query node tree for point (helper function)
fn query_point_node(node: &QuadtreeNode, x: f64, y: f64, results: &mut Vec<usize>) {
    if !node.bounds.contains_point(x, y) {
        return;
    }

    results.extend(&node.items);

    if let Some(children) = &node.children {
        for child in children.iter() {
            query_point_node(child, x, y, results);
        }
    }
}

/// Quadtree spatial index for efficient shape queries
#[derive(Debug)]
pub struct SpatialIndex {
    root: QuadtreeNode,
    max_depth: usize,
    max_items_per_node: usize,
}

impl SpatialIndex {
    /// Create new spatial index
    pub fn new(bounds: Bounds, max_depth: usize, max_items_per_node: usize) -> Self {
        Self {
            root: QuadtreeNode::new_leaf(bounds),
            max_depth,
            max_items_per_node,
        }
    }

    /// Insert item at given bounds
    pub fn insert(&mut self, index: usize, item_bounds: &Bounds) {
        insert_into_node(&mut self.root, index, item_bounds, self.max_depth, self.max_items_per_node);
    }

    /// Query items in given bounds
    pub fn query(&self, query_bounds: &Bounds) -> Vec<usize> {
        let mut results = Vec::new();
        query_node(&self.root, query_bounds, &mut results);
        results
    }

    /// Query items containing a point
    pub fn query_point(&self, x: f64, y: f64) -> Vec<usize> {
        let mut results = Vec::new();
        query_point_node(&self.root, x, y, &mut results);
        results
    }

    /// Clear all items from index
    pub fn clear(&mut self) {
        self.root.items.clear();
        self.root.children = None;
    }

    /// Get memory usage estimate in bytes
    pub fn memory_usage(&self) -> usize {
        estimate_node_size(&self.root)
    }

    /// Get statistics about the index
    pub fn stats(&self) -> SpatialIndexStats {
        let mut stats = SpatialIndexStats {
            total_nodes: 0,
            total_items: 0,
            max_depth_reached: 0,
            avg_items_per_node: 0.0,
        };

        collect_stats(&self.root, &mut stats);

        if stats.total_nodes > 0 {
            stats.avg_items_per_node = stats.total_items as f64 / stats.total_nodes as f64;
        }

        stats
    }
}

/// Estimate node tree size (helper)
fn estimate_node_size(node: &QuadtreeNode) -> usize {
    let mut size = std::mem::size_of::<QuadtreeNode>();
    size += node.items.capacity() * std::mem::size_of::<usize>();

    if let Some(children) = &node.children {
        for child in children.iter() {
            size += estimate_node_size(child);
        }
    }

    size
}

/// Collect statistics (helper)
fn collect_stats(node: &QuadtreeNode, stats: &mut SpatialIndexStats) {
    stats.total_nodes += 1;
    stats.total_items += node.items.len();
    stats.max_depth_reached = stats.max_depth_reached.max(node.depth);

    if let Some(children) = &node.children {
        for child in children.iter() {
            collect_stats(child, stats);
        }
    }
}

/// Statistics for spatial index
#[derive(Debug, Clone)]
pub struct SpatialIndexStats {
    pub total_nodes: usize,
    pub total_items: usize,
    pub max_depth_reached: usize,
    pub avg_items_per_node: f64,
}

impl Default for SpatialIndex {
    fn default() -> Self {
        Self::new(
            Bounds::new(-1000.0, -1000.0, 1000.0, 1000.0),
            8,
            16,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds_creation() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        assert_eq!(bounds.min_x, 0.0);
        assert_eq!(bounds.max_x, 10.0);
        assert_eq!(bounds.width(), 10.0);
        assert_eq!(bounds.height(), 10.0);
    }

    #[test]
    fn test_bounds_center() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let (cx, cy) = bounds.center();
        assert_eq!(cx, 5.0);
        assert_eq!(cy, 5.0);
    }

    #[test]
    fn test_bounds_contains_point() {
        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        assert!(bounds.contains_point(5.0, 5.0));
        assert!(bounds.contains_point(0.0, 0.0));
        assert!(bounds.contains_point(10.0, 10.0));
        assert!(!bounds.contains_point(11.0, 5.0));
        assert!(!bounds.contains_point(5.0, -1.0));
    }

    #[test]
    fn test_bounds_intersection() {
        let b1 = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let b2 = Bounds::new(5.0, 5.0, 15.0, 15.0);
        let b3 = Bounds::new(20.0, 20.0, 30.0, 30.0);

        assert!(b1.intersects(&b2));
        assert!(b2.intersects(&b1));
        assert!(!b1.intersects(&b3));
    }

    #[test]
    fn test_spatial_index_creation() {
        let index = SpatialIndex::new(
            Bounds::new(-100.0, -100.0, 100.0, 100.0),
            8,
            16,
        );
        assert_eq!(index.root.items.len(), 0);
    }

    #[test]
    fn test_spatial_index_insert_and_query() {
        let mut index = SpatialIndex::new(
            Bounds::new(-100.0, -100.0, 100.0, 100.0),
            8,
            16,
        );

        let bounds1 = Bounds::new(0.0, 0.0, 10.0, 10.0);
        let bounds2 = Bounds::new(5.0, 5.0, 15.0, 15.0);

        index.insert(0, &bounds1);
        index.insert(1, &bounds2);

        let results = index.query(&Bounds::new(7.0, 7.0, 12.0, 12.0));
        assert!(!results.is_empty());
    }

    #[test]
    fn test_spatial_index_query_point() {
        let mut index = SpatialIndex::new(
            Bounds::new(-100.0, -100.0, 100.0, 100.0),
            8,
            16,
        );

        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        index.insert(0, &bounds);

        let results = index.query_point(5.0, 5.0);
        assert!(results.contains(&0));

        // Query far away - outside root bounds shouldn't match
        let results2 = index.query_point(150.0, 150.0);
        assert!(!results2.contains(&0));
    }

    #[test]
    fn test_spatial_index_clear() {
        let mut index = SpatialIndex::new(
            Bounds::new(-100.0, -100.0, 100.0, 100.0),
            8,
            16,
        );

        let bounds = Bounds::new(0.0, 0.0, 10.0, 10.0);
        index.insert(0, &bounds);
        assert!(!index.query_point(5.0, 5.0).is_empty());

        index.clear();
        assert!(index.query_point(5.0, 5.0).is_empty());
    }

    #[test]
    fn test_spatial_index_stats() {
        let mut index = SpatialIndex::new(
            Bounds::new(-100.0, -100.0, 100.0, 100.0),
            8,
            16,
        );

        for i in 0..20 {
            let bounds = Bounds::new(
                (i as f64) * 5.0,
                (i as f64) * 5.0,
                (i as f64) * 5.0 + 10.0,
                (i as f64) * 5.0 + 10.0,
            );
            index.insert(i, &bounds);
        }

        let stats = index.stats();
        assert!(stats.total_nodes > 1);
        assert!(stats.total_items >= 20);
    }

    #[test]
    fn test_spatial_index_stress() {
        let mut index = SpatialIndex::new(
            Bounds::new(-10000.0, -10000.0, 10000.0, 10000.0),
            8,
            16,
        );

        // Insert 1000 shapes
        for i in 0..1000 {
            let x = ((i as f64) % 50.0) * 10.0;
            let y = ((i as f64) / 50.0) * 10.0;
            let bounds = Bounds::new(x, y, x + 5.0, y + 5.0);
            index.insert(i, &bounds);
        }

        let stats = index.stats();
        assert!(stats.total_items >= 1000);

        // Query should be much faster than scanning all
        let query_bounds = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let results = index.query(&query_bounds);
        assert!(!results.is_empty());
    }
}
