use regex::Regex;
use serde_json::Value;
use si_level_gen::{level::Level, world::World};
use std::fs::{self, File};

#[test]
fn correct_generation() {
    pub fn world_to_bytes(world: &World) -> [u8; 0x2000] {
        let mut data = [u8::MAX; 0x2000];
        for y in 0..64 {
            for x in 0..64 {
                let tile = world.surface_level.tile(x, y) as u8;
                data[y * 128 + x] = tile;
            }
        }
        for y in 0..32 {
            for x in 0..32 {
                let tile = world.cave_level.tile(x, y) as u8;
                data[y * 128 + x + 64] = tile;
            }
        }
        data
    }

    let regex_filename = Regex::new(r"^(\d+)\.dat$").unwrap();

    let start_positions: Value = serde_json::from_reader(
        File::open("tests/resources/correct_map_data/start_positions.json").unwrap(),
    )
    .unwrap();

    for file in fs::read_dir("tests/resources/correct_map_data")
        .unwrap()
        .map(Result::unwrap)
        .filter(|f| {
            f.path()
                .extension()
                .map_or(false, |extension| extension.eq_ignore_ascii_case("dat"))
        })
    {
        let file_name = file.file_name();
        let captures = &regex_filename
            .captures(file_name.to_str().unwrap())
            .unwrap();
        let seed = captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let correct_map_data = fs::read(file.path().as_path()).unwrap();
        let generated_level = World::generate(seed).unwrap();
        let generated_map_data = world_to_bytes(&generated_level);
        assert!(
            correct_map_data == generated_map_data,
            "map data for seed {seed} is incorrect"
        );

        assert_eq!(
            generated_level.surface_level.start_position().0,
            start_positions[seed.to_string()]["x"]
        );

        assert_eq!(
            generated_level.surface_level.start_position().1,
            start_positions[seed.to_string()]["y"]
        );
    }
}
