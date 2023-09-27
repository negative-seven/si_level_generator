use std::collections::BTreeSet;

use crate::{
    level::{Cave, Level, Surface},
    random::Random,
};
use fixed::types::I16F16;

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum LevelType {
    Surface,
    Cave,
}

pub struct World {
    pub surface_level: Surface,
    pub surface_level_enemies: Option<BTreeSet<(u8, u8)>>,
    pub cave_level: Cave,
    pub cave_level_enemies: Option<BTreeSet<(u8, u8)>>,
}

impl World {
    pub fn generate(seed: u32) -> Result<Self, String> {
        #[allow(clippy::cast_possible_wrap)]
        let mut random = Random::new(seed);
        for _ in 0..256 {
            random.next();
        }

        Ok(World {
            cave_level: Cave::generate(&mut random)?,
            cave_level_enemies: None,
            surface_level: Surface::generate(&mut random)?,
            surface_level_enemies: None,
        })
    }

    /// TODO: untested + the distance-to-player condition is ignored
    pub fn generate_enemies(&mut self, seed: u32) {
        let mut random_initializer = Random::new(seed);
        #[allow(clippy::cast_sign_loss)]
        let mut random = Random::new(random_initializer.next_max(I16F16::MIN).to_bits() as u32);

        self.cave_level_enemies = Some(self.cave_level.generate_enemies(&mut random));
        self.surface_level_enemies = Some(self.surface_level.generate_enemies(&mut random));
    }
}
