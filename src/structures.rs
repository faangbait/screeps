
pub fn init() {

}

pub trait OptimalHarvest {
    fn harvest_work_per_sec(self: &Self) -> u8;
}

impl OptimalHarvest for screeps::Source {
    fn harvest_work_per_sec(self: &Self) -> u8 {
        let raw = match self.ticks_to_regeneration().checked_sub(1).unwrap_or_else(||0) {
            0..=1 => 5.,
            2..=5 => 5_f32.max((self.energy() as f32 / (self.ticks_to_regeneration() - 1) as f32) / 2.),
            6..=30 => 3_f32.max((self.energy() as f32 / (self.ticks_to_regeneration() -1) as f32) / 2.),
            _ => (self.energy() as f32 / (self.ticks_to_regeneration() - 1) as f32) / 2.
        };
        raw.ceil().min(12.) as u8
    }
}
