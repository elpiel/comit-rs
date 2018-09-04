#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Blocks(u32);

impl Blocks {
    pub const fn new(num_blocks: u32) -> Self {
        Blocks(num_blocks)
    }
}

impl From<u32> for Blocks {
    fn from(num: u32) -> Self {
        Blocks::new(num)
    }
}

impl From<Blocks> for u32 {
    fn from(blocks: Blocks) -> u32 {
        blocks.0
    }
}
