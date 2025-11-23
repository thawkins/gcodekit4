use super::visualizer_2d::{GCodeCommand, Point2D};
use std::fmt::Write;

#[derive(Debug, Default, Clone)]
pub struct ToolpathCache {
    content_hash: u64,
    commands: Vec<GCodeCommand>,
    cached_path: String,
    cached_rapid_path: String,
}

impl ToolpathCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn needs_update(&self, new_hash: u64) -> bool {
        self.content_hash != new_hash || self.commands.is_empty()
    }

    pub fn update(&mut self, new_hash: u64, commands: Vec<GCodeCommand>) {
        self.content_hash = new_hash;
        self.commands = commands;
        self.rebuild_paths();
    }

    pub fn commands(&self) -> &[GCodeCommand] {
        &self.commands
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn toolpath_svg(&self) -> &str {
        &self.cached_path
    }

    pub fn rapid_svg(&self) -> &str {
        &self.cached_rapid_path
    }

    fn rebuild_paths(&mut self) {
        self.cached_path.clear();
        self.cached_rapid_path.clear();

        if self.commands.is_empty() {
            return;
        }

        self.cached_path.reserve(self.commands.len() * 25);
        self.cached_rapid_path.reserve(self.commands.len() * 10);

        let mut last_pos: Option<Point2D> = None;

        for cmd in &self.commands {
            match cmd {
                GCodeCommand::Move { from, to, rapid } => {
                    if *rapid {
                        let _ = write!(
                            self.cached_rapid_path,
                            "M {:.2} {:.2} L {:.2} {:.2} ",
                            from.x, -from.y, to.x, -to.y
                        );
                        last_pos = None;
                        continue;
                    }

                    if last_pos.is_none() {
                        let _ = write!(self.cached_path, "M {:.2} {:.2} ", from.x, -from.y);
                    }
                    let _ = write!(self.cached_path, "L {:.2} {:.2} ", to.x, -to.y);
                    last_pos = Some(*to);
                }
                GCodeCommand::Arc {
                    from,
                    to,
                    center,
                    clockwise,
                } => {
                    let radius = ((from.x - center.x).powi(2) + (from.y - center.y).powi(2)).sqrt();

                    if last_pos.is_none() {
                        let _ = write!(self.cached_path, "M {:.2} {:.2} ", from.x, -from.y);
                    }

                    let sweep = if *clockwise { 0 } else { 1 };

                    use std::f32::consts::PI;
                    let start_angle = (from.y - center.y).atan2(from.x - center.x);
                    let end_angle = (to.y - center.y).atan2(to.x - center.x);
                    let mut angle_diff = if *clockwise {
                        start_angle - end_angle
                    } else {
                        end_angle - start_angle
                    };

                    while angle_diff < 0.0 {
                        angle_diff += 2.0 * PI;
                    }
                    while angle_diff >= 2.0 * PI {
                        angle_diff -= 2.0 * PI;
                    }

                    let large_arc = if angle_diff > PI { 1 } else { 0 };

                    let _ = write!(
                        self.cached_path,
                        "A {:.2} {:.2} 0 {} {} {:.2} {:.2} ",
                        radius, radius, large_arc, sweep, to.x, -to.y
                    );

                    last_pos = Some(*to);
                }
            }
        }
    }
}
