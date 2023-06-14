use crate::network::Network;
use rand::rngs::ThreadRng;
use rand::Rng;
use rstar::{PointDistance, RTreeObject, AABB};
use sdl2::pixels::Color;

pub const COLOR_MUTATION_RATE: f32 = 0.0;
pub const COLOR_MUTATION_VAL: i32 = 10;

pub struct Cell {
    pub network: Network,
    pub x: i32,
    pub y: i32,
    pub ticks_since_food: u32,
    pub alive: bool,
    pub color: Color,
}
#[derive(Debug, Clone)]
pub struct Food {
    pub x: i32,
    pub y: i32,
}

/*

Network contains the entire board

*/
impl Cell {
    pub fn new(x: i32, y: i32, alive: bool, rng: &mut ThreadRng) -> Self {
        let r = rng.gen_range(0..255);
        let g = rng.gen_range(0..255);
        let b = rng.gen_range(0..255);
        let cell = Self {
            network: Network::new(vec![8, 12, 8, 2], rng),
            x: x,
            y: y,
            ticks_since_food: 0,
            alive: alive,
            color: Color::RGB(r, g, b),
        };

        return cell;
    }

    pub fn inherit_from(parent: &Cell, new_x: i32, new_y: i32, rng: &mut ThreadRng) -> Cell {
        let mut color = parent.color.clone();
        if rng.gen_range(0.0..=1.0) < COLOR_MUTATION_RATE {
            let index = rng.gen_range(0..3);
            match index {
                0 => {
                    color.r = (color.r as i32
                        + if rng.gen_range(0..=1) != 0 {
                            COLOR_MUTATION_VAL
                        } else {
                            -COLOR_MUTATION_VAL
                        })
                    .abs() as u8;
                }
                1 => {
                    color.g = (color.g as i32
                        + if rng.gen_range(0..=1) != 0 {
                            COLOR_MUTATION_VAL
                        } else {
                            -COLOR_MUTATION_VAL
                        })
                    .abs() as u8;
                }
                2 => {
                    color.b = (color.b as i32
                        + if rng.gen_range(0..=1) != 0 {
                            COLOR_MUTATION_VAL
                        } else {
                            -COLOR_MUTATION_VAL
                        })
                    .abs() as u8;
                }
                _ => {}
            }
        }

        let cell = Cell {
            network: Network::inherit_from(&parent.network, rng),
            x: new_x,
            y: new_y,
            ticks_since_food: 0,
            alive: true,
            color: color,
        };

        return cell;
    }
}

impl Food {
    pub fn new(x: i32, y: i32) -> Self {
        let food = Self { x: x, y: y };

        return food;
    }
}

impl RTreeObject for Food {
    type Envelope = AABB<[i32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x, self.y])
    }
}

impl PointDistance for Food {
    fn distance_2(&self, point: &[i32; 2]) -> i32 {
        let d_x = self.x - point[0];
        let d_y = self.y - point[1];
        let distance = d_x.abs() + d_y.abs();
        distance
    }

    // This implementation is not required but more efficient since it
    // omits the calculation of a square root
    fn contains_point(&self, point: &[i32; 2]) -> bool {
        let d_x = self.x - point[0];
        let d_y = self.y - point[1];
        d_x == 0 && d_y == 0
    }
}

impl RTreeObject for Cell {
    type Envelope = AABB<[i32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x, self.y])
    }
}

impl PointDistance for Cell {
    fn distance_2(&self, point: &[i32; 2]) -> i32 {
        let d_x = self.x - point[0];
        let d_y = self.y - point[1];
        let distance = d_x.abs() + d_y.abs();
        distance
    }

    // This implementation is not required but more efficient since it
    // omits the calculation of a square root
    fn contains_point(&self, point: &[i32; 2]) -> bool {
        let d_x = self.x - point[0];
        let d_y = self.y - point[1];
        d_x == 0 && d_y == 0
    }
}
