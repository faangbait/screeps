pub struct World {
    size: u32
}

impl World {
    pub fn new(size: u32) -> Self { Self { 
        size: screeps::game::map::get_world_size()
    } }
}
