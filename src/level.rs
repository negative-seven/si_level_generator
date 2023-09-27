use crate::{random::Random, tile::Tile};
use fixed::types::I16F16;
use std::{cmp::max, collections::BTreeSet};

pub trait Level<const SIZE: usize>: Sized {
    fn terrain_tile_types() -> (Tile, Tile, Tile, Tile, Tile);
    fn ladder_surrounding_tile_type() -> Tile;
    fn tile<T>(&self, x: T, y: T) -> Tile
    where
        T: Into<usize>;
    fn tile_mut<T>(&mut self, x: T, y: T) -> &mut Tile
    where
        T: Into<usize>;
    fn generate(random: &mut Random) -> Result<Self, String>;

    fn try_generate_terrain(
        random: &mut Random,
        min_tile_1_count: u16,
        min_tile_3_count: u16,
        min_tile_4_count: u16,
    ) -> Option<[[Tile; SIZE]; SIZE]> {
        let (tile_0, tile_1, tile_2, tile_3, tile_4) = Self::terrain_tile_types();

        let cur =
            generate_noise::<SIZE>(random, I16F16::from_num(0.9), I16F16::from_num(0.2), SIZE);
        let cur2 = generate_noise::<SIZE>(random, I16F16::from_num(0.9), I16F16::from_num(0.4), 8);
        let cur3 = generate_noise::<SIZE>(random, I16F16::from_num(0.9), I16F16::from_num(0.3), 8);
        let cur4 = generate_noise::<SIZE>(random, I16F16::from_num(0.8), I16F16::from_num(1.1), 4);

        let mut tile_1_count = 0;
        let mut tile_3_count = 0;
        let mut tile_4_count = 0;

        let mut tiles = [[Tile::None; SIZE]; SIZE];

        for i in 0..SIZE {
            for j in 0..SIZE {
                let v = (cur[i][j] - cur2[i][j]).abs();
                let v2 = (cur[i][j] - cur3[i][j]).abs();
                let v3 = (cur[i][j] - cur4[i][j]).abs();
                let dist = max(
                    (I16F16::from_num(i) / I16F16::from_num(SIZE) - I16F16::from_num(0.5)).abs()
                        * 2,
                    (I16F16::from_num(j) / I16F16::from_num(SIZE) - I16F16::from_num(0.5)).abs()
                        * 2,
                );
                let coast = v * 4 - dist * dist * dist * dist * 4;

                let id = if coast < I16F16::from_num(-1.3) {
                    Tile::Water
                } else if coast > I16F16::from_num(0.3) {
                    if v2 > I16F16::from_num(0.5) {
                        tile_3_count += 1;
                        tile_3
                    } else if coast > I16F16::from_num(0.6) {
                        if v3 > I16F16::from_num(0.5) {
                            tile_4_count += 1;
                            tile_4
                        } else {
                            tile_2
                        }
                    } else {
                        tile_1_count += 1;
                        tile_1
                    }
                } else {
                    tile_0
                };

                tiles[i][j] = id;
            }
        }

        if tile_1_count >= min_tile_1_count
            && tile_3_count >= min_tile_3_count
            && tile_4_count >= min_tile_4_count
        {
            Some(tiles)
        } else {
            None
        }
    }

    fn place_ladder(&mut self) {
        self.place_surrounded_tile(
            (SIZE / 2).try_into().unwrap(),
            (SIZE / 2).try_into().unwrap(),
            Tile::Ladder,
            Self::ladder_surrounding_tile_type(),
        );
    }

    fn place_surrounded_tile(&mut self, x: u8, y: u8, center_tile: Tile, surrounding_tile: Tile) {
        for i in x - 1..x + 2 {
            for j in y - 1..y + 2 {
                *self.tile_mut(i, j) = surrounding_tile;
            }
        }
        *self.tile_mut(x, y) = center_tile;
    }

    fn choose_random_sand_or_grass_position(&self, random: &mut Random) -> Option<(u8, u8)> {
        for _ in 0..501 {
            let x = ((SIZE / 8) + random.next_uint_max(SIZE * 6 / 8))
                .try_into()
                .unwrap();
            let y = ((SIZE / 8) + random.next_uint_max(SIZE * 6 / 8))
                .try_into()
                .unwrap();
            let tile = self.tile(x, y);
            if tile == Tile::Sand || tile == Tile::Grass {
                return Some((x, y));
            }
        }
        None
    }

    /// TODO: untested + the distance-to-player condition is ignored
    fn generate_enemies(&self, random: &mut Random) -> BTreeSet<(u8, u8)> {
        let mut enemies = BTreeSet::new();
        for x in 0..SIZE {
            for y in 0..SIZE {
                if random.next_max(I16F16::from_num(100)) >= I16F16::from_num(3) {
                    continue;
                }

                match self.tile(x, y) {
                    Tile::Water
                    | Tile::Stone
                    | Tile::Tree
                    | Tile::Iron
                    | Tile::Gold
                    | Tile::Gem => continue,
                    _ => (),
                }

                enemies.insert((x.try_into().unwrap(), y.try_into().unwrap()));
            }
        }

        enemies
    }
}

pub struct Surface {
    tiles: [[Tile; Self::SIZE]; Self::SIZE],
    artifacts: [(u8, u8, Tile); 4],
    start_position: (u8, u8),
}

impl Surface {
    const SIZE: usize = 64;

    #[must_use]
    pub fn start_position(&self) -> (u8, u8) {
        self.start_position
    }

    #[must_use]
    pub fn artifacts(&self) -> &[(u8, u8, Tile); 4] {
        &self.artifacts
    }

    fn choose_start_position(&mut self, random: &mut Random) -> Result<(), String> {
        if let Some(position) = self.choose_random_sand_or_grass_position(random) {
            self.start_position = position;
            Ok(())
        } else {
            Err("failed to choose start position".to_string())
        }
    }

    fn place_artifacts(&mut self, random: &mut Random) -> Result<(), String> {
        let mut artifacts = [(0, 0, Tile::None); 4];
        for artifact in &mut artifacts {
            if let Some((ax, ay)) = self.choose_random_sand_or_grass_position(random) {
                let new_tile = self.tile(ax, ay).to_tile_with_artifact().unwrap();
                *self.tile_mut(ax, ay) = new_tile;
                *artifact = (ax, ay, new_tile);
            } else {
                return Err("failed to choose tile for artifact".to_string());
            }
        }
        self.artifacts = artifacts;
        Ok(())
    }
}

impl Level<{ Self::SIZE }> for Surface {
    fn terrain_tile_types() -> (Tile, Tile, Tile, Tile, Tile) {
        (
            Tile::Water,
            Tile::Sand,
            Tile::Grass,
            Tile::Stone,
            Tile::Tree,
        )
    }

    fn ladder_surrounding_tile_type() -> Tile {
        Tile::Stone
    }

    fn tile<T>(&self, x: T, y: T) -> Tile
    where
        T: Into<usize>,
    {
        self.tiles[x.into()][y.into()]
    }

    fn tile_mut<T>(&mut self, x: T, y: T) -> &mut Tile
    where
        T: Into<usize>,
    {
        &mut self.tiles[x.into()][y.into()]
    }

    fn generate(random: &mut Random) -> Result<Self, String> {
        let tiles;
        loop {
            let t = Self::try_generate_terrain(random, 0, 30, 30);
            if let Some(result) = t {
                tiles = result;
                break;
            }
        }

        let mut level = Self {
            tiles,
            artifacts: [(0, 0, Tile::None); 4],
            start_position: (0, 0),
        };

        level.place_ladder();
        level.choose_start_position(random)?;
        level.place_artifacts(random)?;

        Ok(level)
    }
}

pub struct Cave {
    tiles: [[Tile; Self::SIZE]; Self::SIZE],
}

impl Cave {
    const SIZE: usize = 32;

    fn place_boss_entrance(&mut self, random: &mut Random) {
        let mut boss_x = (random.next_max(I16F16::from_num(Self::SIZE - 2)))
            .floor()
            .to_num::<usize>()
            + 1;
        let mut boss_y = if random.next() > I16F16::from_num(0.5) {
            1
        } else {
            Self::SIZE - 2
        };
        if random.next() > 0.5 {
            (boss_x, boss_y) = (boss_y, boss_x);
        }
        self.place_surrounded_tile(
            boss_x.try_into().unwrap(),
            boss_y.try_into().unwrap(),
            Tile::BossLadder,
            Tile::Gem,
        );
    }
}

impl Level<{ Self::SIZE }> for Cave {
    fn terrain_tile_types() -> (Tile, Tile, Tile, Tile, Tile) {
        (Tile::Stone, Tile::Iron, Tile::Sand, Tile::Gold, Tile::Gem)
    }

    fn ladder_surrounding_tile_type() -> Tile {
        Tile::Sand
    }

    fn tile<T>(&self, x: T, y: T) -> Tile
    where
        T: Into<usize>,
    {
        self.tiles[x.into()][y.into()]
    }

    fn tile_mut<T>(&mut self, x: T, y: T) -> &mut Tile
    where
        T: Into<usize>,
    {
        &mut self.tiles[x.into()][y.into()]
    }

    fn generate(random: &mut Random) -> Result<Self, String> {
        let tiles;
        loop {
            if let Some(result) = Self::try_generate_terrain(random, 30, 20, 15) {
                tiles = result;
                break;
            }
        }

        let mut level = Self { tiles };

        level.place_ladder();

        if level.choose_random_sand_or_grass_position(random).is_none() {
            return Err(
                "failed to choose discarded tile while generating cave"
                    .to_string(),
            );
        }

        level.place_boss_entrance(random);

        Ok(level)
    }
}

fn generate_noise<const SIZE: usize>(
    random: &mut Random,
    starting_scale: I16F16,
    scale_multiplier: I16F16,
    features_step: usize,
) -> [[I16F16; SIZE]; SIZE] {
    let mut noise = [[I16F16::from_num(0.5); SIZE]; SIZE];

    let mut step = SIZE;
    let mut random_scale = starting_scale;
    while step > 1 {
        let adjusted_random_scale = if step == features_step {
            I16F16::from_num(1)
        } else {
            random_scale
        };
        let mut random = || (random.next() - I16F16::from_num(0.5)) * adjusted_random_scale;

        for x in (0..SIZE - step).step_by(step) {
            for y in (0..SIZE - step).step_by(step) {
                noise[x + step / 2][y] =
                    (noise[x][y] + noise[x + step][y]) * I16F16::from_num(0.5) + random();
                noise[x][y + step / 2] =
                    (noise[x][y] + noise[x][y + step]) * I16F16::from_num(0.5) + random();
            }

            // special case: bottom row
            noise[x + step / 2][SIZE - step] =
                (noise[x][SIZE - step] + noise[x + step][SIZE - step]) * I16F16::from_num(0.5)
                    + random();
            noise[x][SIZE - step / 2] =
                (noise[x][SIZE - step] + I16F16::from_num(0.5)) * I16F16::from_num(0.5) + random();
        }

        // special case: right column
        for y in (0..SIZE - step).step_by(step) {
            noise[SIZE - step / 2][y] =
                (noise[SIZE - step][y] + I16F16::from_num(0.5)) * I16F16::from_num(0.5) + random();
            noise[SIZE - step][y + step / 2] =
                (noise[SIZE - step][y] + noise[SIZE - step][y + step]) * I16F16::from_num(0.5)
                    + random();
        }

        // special case: bottom right corner
        noise[SIZE - step / 2][SIZE - step] =
            (noise[SIZE - step][SIZE - step] + I16F16::from_num(0.5)) * I16F16::from_num(0.5)
                + random();
        noise[SIZE - step][SIZE - step / 2] =
            (noise[SIZE - step][SIZE - step] + I16F16::from_num(0.5)) * I16F16::from_num(0.5)
                + random();

        for x in (0..SIZE - step).step_by(step) {
            for y in (0..SIZE - step).step_by(step) {
                noise[x + step / 2][y + step / 2] = (noise[x][y]
                    + noise[x + step][y]
                    + noise[x][y + step]
                    + noise[x + step][y + step])
                    * I16F16::from_num(0.25)
                    + random();
            }

            // special case: bottom row
            noise[x + step / 2][SIZE - step / 2] =
                (noise[x][SIZE - step] + noise[x + step][SIZE - step] + I16F16::ONE)
                    * I16F16::from_num(0.25)
                    + random();
        }

        // special case: right column
        for y in (0..SIZE - step).step_by(step) {
            noise[SIZE - step / 2][y + step / 2] =
                (noise[SIZE - step][y] + noise[SIZE - step][y + step] + I16F16::ONE)
                    * I16F16::from_num(0.25)
                    + random();
        }

        // special case: bottom right corner
        noise[SIZE - step / 2][SIZE - step / 2] =
            (noise[SIZE - step][SIZE - step] + I16F16::from_num(1.5)) * I16F16::from_num(0.25)
                + random();

        step /= 2;
        random_scale *= scale_multiplier;
    }
    noise
}
