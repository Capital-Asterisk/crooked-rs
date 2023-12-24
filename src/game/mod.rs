
use glam::{vec2, Vec2, UVec2};
use crate::lgrn;

pub const TILE_SIZE: Vec2 = vec2(64.0, 64.0);

lgrn::id_type!(DroneId);
lgrn::id_type!(BulletId);

#[derive(Default)]
pub struct GameMain
{
    pub world_size:     UVec2,
    pub player_pos:     Vec2,
    pub player_facing:  i8,
    pub hop_count:      u64,

    pub rail:           Vec<Rail>,

    pub drone_ids:      lgrn::IdReg<DroneId>,
    pub drone_pos:      Vec<Vec2>,
    pub drone_data:     Vec<DroneData>,
    pub drone_by_x:     Vec<(DroneId, f32)>,

    pub bullet_ids:     lgrn::IdReg<BulletId>,
    pub bullet_pos:     Vec<Vec2>,
    pub bullet_data:    Vec<BulletData>,

    pub remove_drones:  Vec<DroneId>,
    pub remove_bullets: Vec<BulletId>

}

pub struct Controls {
    pub walk: Vec2
}

pub enum Dir { Right, Down, Left, Up }
pub enum Bend { Forward, Right, Left }

pub struct Rail {
    pub pos:    UVec2,
    pub dir:    Dir,
    pub bend:   Bend
}


#[derive(Clone)]
pub struct DroneData {
    pub rail_idx:   usize,
    pub rail_pos:   f32,
    pub speed:      f32
}

#[derive(Clone)]
pub struct BulletData {
    pub dir:    Vec2,
    pub speed:  f32,
    pub travel: f32,
    pub travel_max: f32
}


pub fn rot_cw_90(a: Vec2) -> Vec2 { vec2(-a.y, a.x) }

pub fn rot_ccw_90(a: Vec2) -> Vec2 { vec2(a.y, -a.x) }


pub fn line_segment_vs_line_intersect(a_pos: (Vec2, Vec2), b_pos: Vec2, b_dir: Vec2) -> bool {

    // based on https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection#Given_two_points_on_each_line_segment

    let x1 = a_pos.0.x;
    let y1 = a_pos.0.y;
    let x2 = a_pos.1.x;
    let y2 = a_pos.1.y;

    let x3 = b_pos.x;
    let y3 = b_pos.y;

    let t =   ( (y1-y3)*(b_dir.x) - (x1-x3)*(b_dir.y) )
            / ( (y1-y2)*(b_dir.x) - (x1-x2)*(b_dir.y) );

    0.0 <= t && t <= 1.0
}


#[cfg(test)]
mod tests {

    use std::ops::Not;

    use super::*;

    #[test]
    fn test_line_intersect() {
        assert!(line_segment_vs_line_intersect((vec2(5.0, 5.0), vec2(10.0, 10.0)), vec2(6.0, 0.0), vec2(0.0, 1.0)));

        assert!(line_segment_vs_line_intersect((vec2(5.0, 5.0), vec2(10.0, 10.0)), vec2(11.0, 0.0), vec2(0.0, 1.0)).not());

        assert!(line_segment_vs_line_intersect((vec2(5.0, 5.0), vec2(10.0, 10.0)), vec2(4.0, 0.0), vec2(0.0, 1.0)).not());
    }
}

