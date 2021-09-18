pub struct Settings {
    draw_distance: usize,
}

impl Settings {
    pub fn new(draw_distance: usize) -> Self {
        Self {
            draw_distance
        }
    }

    pub fn get_draw_distance(&self) -> usize {
        self.draw_distance
    }
}