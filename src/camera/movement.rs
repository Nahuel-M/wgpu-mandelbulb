#[derive(Default, Debug)]
pub struct Movement {
    pub forward: f32,
    pub right:   f32,
    pub up:      f32,

    pub pitch: f32,
    pub yaw:   f32,

    pub speed: f32,
}
