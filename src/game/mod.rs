
use glam::{vec2, Mat4, Vec2, UVec2};
use crate::lgrn;

pub const TILE_SIZE: Vec2 = vec2(64.0, 64.0);

struct EnemyId(usize);
lgrn::impl_id_type!(EnemyId);

#[derive(Default)]
pub struct GameMain
{
    pub world_size: UVec2,
    pub player_pos: Vec2,
    pub player_facing: i8,
    pub hop_count: u64,

    pub rail: Vec<Rail>
}

pub struct Controls
{
    pub walk: Vec2
}

pub enum Dir { Right, Up, Left, Down }

pub struct Rail
{
    pub pos: UVec2,
    pub dir: Dir,
    pub bend: bool
}

