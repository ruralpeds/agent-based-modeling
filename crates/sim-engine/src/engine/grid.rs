use crate::rng::Mulberry32;

/// 2D resource grid (grass for prey to eat)
pub struct Grid<T> {
    pub width:  usize,
    pub height: usize,
    pub data:   Vec<T>,
}

impl Grid<f32> {
    pub fn new_grass(width: usize, height: usize, rng: &mut Mulberry32) -> Self {
        let data = (0..width * height)
            .map(|_| rng.next_f32())
            .collect();
        Self { width, height, data }
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn get_at(&self, x: f32, y: f32) -> f32 {
        let gx = (x as usize).min(self.width - 1);
        let gy = (y as usize).min(self.height - 1);
        self.data[self.idx(gx, gy)]
    }

    /// Agent eats at position — returns energy gained
    pub fn eat_at(&mut self, x: f32, y: f32, max_eat: f32) -> f32 {
        let gx = (x as usize).min(self.width - 1);
        let gy = (y as usize).min(self.height - 1);
        let i = self.idx(gx, gy);
        let gained = self.data[i].min(max_eat);
        self.data[i] -= gained;
        gained
    }

    /// Regrow grass each tick
    pub fn regrow(&mut self, rate: f32, rng: &mut Mulberry32) {
        for v in &mut self.data {
            if *v < 1.0 {
                *v += rate * rng.next_f32();
                if *v > 1.0 {
                    *v = 1.0;
                }
            }
        }
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.data
    }
}
