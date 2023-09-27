#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Tile {
    None = -1,
    Water = 0,
    Sand = 1,
    Grass = 2,
    Stone = 3,
    Tree = 4,
    Iron = 8,
    Gold = 9,
    Gem = 10,
    Ladder = 11,
    SandWithArtifact = 12,
    GrassWithArtifact = 13,
    BossLadder = 14,
}

impl Tile {
    #[must_use]
    pub fn to_tile_with_artifact(&self) -> Option<Self> {
        match self {
            Self::Sand => Some(Self::SandWithArtifact),
            Self::Grass => Some(Self::GrassWithArtifact),
            _ => None,
        }
    }
}
