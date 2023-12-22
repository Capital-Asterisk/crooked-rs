
use glam::{vec2, Vec2, UVec2};
use crate::lgrn;

pub const TILE_SIZE: Vec2 = vec2(64.0, 64.0);

lgrn::id_type!(DroneId);

#[derive(Default)]
pub struct GameMain
{
    pub world_size: UVec2,
    pub player_pos: Vec2,
    pub player_facing: i8,
    pub hop_count: u64,

    pub rail: Vec<Rail>,

    pub drone_ids: lgrn::IdReg<DroneId>,
    pub drone_pos: Vec<Vec2>,
    pub drone_data: Vec<DroneData>
}

pub struct Controls
{
    pub walk: Vec2
}

#[derive(Clone)]
pub struct DroneData
{
    pub rail_idx: usize,
    pub rail_pos: f32,
    pub speed: f32
}

pub enum Dir { Right, Down, Left, Up }
pub enum Bend { Forward, Right, Left }


pub struct Rail
{
    pub pos: UVec2,
    pub dir: Dir,
    pub bend: Bend
}

