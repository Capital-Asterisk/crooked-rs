pub mod items;
pub use items::*;

use std::iter;

use glam::{vec2, Vec2, UVec2, IVec2, ivec2, uvec2, Mat2};
use std::{collections::BTreeMap, default};
use crate::lgrn;

use std::ops::Not;

pub const TILE_SIZE: Vec2 = vec2(64.0, 64.0);

lgrn::id_type!(DroneId);
lgrn::id_type!(BulletId);
lgrn::id_type!(MachineId);

pub const PLAYER_GUN_AMMO_MAX: u32 = 132;

#[derive(Default)]
pub struct GameMain {
    pub time_us:        u64,
    pub world_size:     UVec2,
    pub player_pos:     Vec2,
    pub player_facing:  i8,
    pub player_gun_cooldown: f32,
    pub player_gun_consecutive: u32,
    pub player_gun_ammo: u32,
    pub hop_count:      u64,

    pub rail:           Vec<Rail>,
    pub rail_by_tile:   BTreeMap<(u8, u8), u32>,

    pub drone_ids:      lgrn::IdReg<DroneId>,
    pub drone_pos:      Vec<Vec2>,
    pub drone_data:     Vec<Drone>,
    pub drone_by_x:     Vec<(DroneId, f32)>,

    pub drone_per_second: f32,
    pub drone_timer:    f32,

    pub bullet_ids:     lgrn::IdReg<BulletId>,
    pub bullet_pos:     Vec<Vec2>,
    pub bullet_data:    Vec<Bullet>,

    pub remove_drones:  Vec<DroneId>,
    pub remove_bullets: Vec<BulletId>,

    pub itemtype_data:  Vec<ItemType>,

    pub feral_ids:      lgrn::IdReg<FeralItemId>,
    pub feral_data:     Vec<FeralItem>,
    pub feral_by_tile:  BTreeMap<(u8, u8), FeralItemId>,

    pub mach_ids:       lgrn::IdReg<MachineId>,
    pub mach_data:      Vec<Machine>,
    pub mach_by_tile:   BTreeMap<(u8, u8), MachineId>,

    pub tool:           ToolMode
}

#[derive(Default)]
pub struct ItemType {
    pub sprite:         (Vec2, Vec2),
    pub stackable:      u32,
    pub name:           &'static str,
    pub desc:           &'static str
}

pub enum ToolMode {
    Construct(Drag),
    GunPod,
    //Build
}

impl Default for ToolMode {
    fn default() -> Self { ToolMode::GunPod }
}

pub enum Drag {
     None,
     Item(ItemSlot),
     Machine(MachineId)
}

impl Default for Drag {
    fn default() -> Self { Drag::None }
}

pub struct Controls {
    pub walk: Vec2
}

#[derive(Clone, Copy, Debug)]
pub enum Dir { Right, Down, Left, Up }

#[derive(Clone, Copy, Debug)]
pub enum Bend { Forward, Right, Left }

#[derive(Clone, Copy)]
pub struct Rail {
    pub pos:    UVec2,
    pub dir:    Dir,
    pub bend:   Bend
}


#[derive(Clone)]
pub struct Drone {
    pub rail_idx:   usize,
    pub rail_pos:   f32,
    pub speed:      f32
}

#[derive(Clone)]
pub struct Bullet {
    pub dir:    Vec2,
    pub speed:  f32,
    pub travel: f32,
    pub travel_max: f32
}

#[derive(Clone)]
pub struct Machine {
    pub pos:    Option<UVec2>,
    pub dir:    Dir,
    pub spec:   MachineSpec
}
impl Default for Machine {
    fn default() -> Self {
        Machine{ pos: None, dir: Dir::Right, spec: MachineSpec::None }
    }
}

#[derive(Clone)]
pub enum MachineSpec {
    None,
    Turret      {ammo: u32, can_fire_time_us: u64},
    Conveyor    {item: ItemSlot, filter: bool, can_move_time_us: u64, can_dump_time_us: u64},
}

pub fn regen_rail_by_tile(rail: &Vec<Rail>, rail_by_tile: &mut BTreeMap<(u8, u8), u32>) {
    rail_by_tile.clear();
    for (i, r) in rail.iter().enumerate()  {
        rail_by_tile.insert((r.pos.x as u8, r.pos.y as u8), i as u32);
    }
}

pub fn world_obfuscate(center: UVec2, rail: &mut Vec<Rail>, world_size: UVec2) -> bool {

    if     (2 > center.x) || (center.x > (world_size.x-4))
        || (2 > center.y) || (center.y > (world_size.y-4)) {
        return false;
    }

    let mut enter_idx: Option<usize> = None;
    let mut exit_idx: Option<usize> = None;
    let mut prev_inside = false;

    let tl = center - uvec2(2, 2);
    let br = center + uvec2(3, 3);

    let mut obstacles:  [[u8; 5]; 5] =
        [[0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0]  ];

    for (i, rail) in rail.iter().enumerate() {
        let rail_inside =    tl.x <= rail.pos.x && rail.pos.x < br.x
                            && tl.y <= rail.pos.y && rail.pos.y < br.y;

        if rail_inside {
            let rpob = rail.pos - tl;
            obstacles[rpob.y as usize][rpob.x as usize] = 1;
        }

        if prev_inside.not() && rail_inside {
            enter_idx = Some(i);
            exit_idx = None;
        } else if prev_inside && rail_inside.not() {
            exit_idx = Some(i);
        }

        prev_inside = rail_inside;
    }

    if enter_idx.is_none() {
        return false;
    }

    if let None = exit_idx { exit_idx = Some(rail.len()); }

    let enter_idx = enter_idx.unwrap();
    let exit_idx = exit_idx.unwrap();

    for rail in &rail[enter_idx..exit_idx] {
        let rpob = rail.pos - tl;
        obstacles[rpob.y as usize][rpob.x as usize] = 0;
    }

    let mut path: Vec<Dir> = Default::default();
    path.reserve(25);

    let enter_rail_dir = if enter_idx == 0 { Dir::Right } else { rail[enter_idx-1].dir };
    let enter_dir = if enter_idx == 0 { Dir::Right } else
    {
        match rail[enter_idx-1].bend {
            Bend::Forward => enter_rail_dir,
            Bend::Right   => vec2_to_dir(rot_cw_90(dir_to_vec2(&enter_rail_dir))),
            Bend::Left    => vec2_to_dir(rot_ccw_90(dir_to_vec2(&enter_rail_dir)))
        }
    };

    let mut success = false;

    let obf_start_pos = rail[enter_idx].pos - tl;
    let obf_end_pos   = rail[exit_idx-1].pos - tl;

    println!("{}->{}", obf_start_pos, obf_end_pos);
    for _ in 0..50 {
        path.clear();
        success = obfuscate(obstacles, obf_start_pos, enter_dir, obf_end_pos, &mut path);
        if success {
            break;
        }
    }
    if success {
        path.push(rail[exit_idx].dir);

        let mut prev_dir = enter_dir;
        let mut prev_diri = dir_to_ivec2(&prev_dir);
        let mut cpos = rail[enter_idx].pos.as_ivec2();

        let dont_care_rail = Rail{pos: uvec2(0, 0), dir: Dir::Right, bend: Bend::Forward};
        rail.splice(enter_idx..exit_idx, iter::repeat(dont_care_rail).take(path.len()));

        for (i, dir) in path.iter().enumerate() {
            let diri = dir_to_ivec2(&dir);

            let bend = match prev_diri.x*diri.y - prev_diri.y*diri.x { // cross product z
                -1 => Bend::Left,
                0 => Bend::Forward,
                1 => Bend::Right,
                _ => panic!()
            };

            rail[enter_idx + i] = Rail { pos: cpos.as_uvec2(), dir: prev_dir, bend};

            prev_dir = dir.clone();
            prev_diri = diri;

            cpos += diri;
        }


    }

    return true;
}

pub fn obfuscate(obstacles: [[u8; 5]; 5], start: UVec2, start_dir: Dir, end: UVec2, out: &mut Vec<Dir>) -> bool {

    use macroquad::rand as mq;

    const CENTER: Vec2 = vec2(2.0, 2.0);

    let mut visited: [[u8; 5]; 5] = [[0; 5]; 5];
    let mut dirs:    [[Dir; 5]; 5]  = [[Dir::Right; 5]; 5];

    let mut steps = 0;
    let mut pos_x:usize = start.x as usize;
    let mut pos_y:usize = start.y as usize;

    let mut prev_dir = dir_to_ivec2(&start_dir).as_vec2();

    let endf = end.as_vec2();

    loop {
        let mut d = vec2(0.0, 0.0);

        let posf = vec2(pos_x as f32, pos_y as f32);

        // nudge towards center
        if steps < 3 {
            d += 0.75 * (CENTER - posf).normalize();
        };

        if steps < 5 {
            // nudge away from end
            d -= 0.5 * (endf - posf).normalize();
        } else {
            // nudge towards end
            d += 0.75 * (endf - posf).normalize();
        }

        // tendancy to stay straight
        d += 0.6 * prev_dir;

        // wall collisions
        if pos_x == 0 && d.x < 0.0 { d.x = 0.0; }
        if pos_x == 4 && d.x > 0.0 { d.x = 0.0; }
        if pos_y == 0 && d.y < 0.0 { d.y = 0.0; }
        if pos_y == 4 && d.y > 0.0 { d.y = 0.0; }

        visited[pos_y][pos_x] = 1;

        let mut success = false;

        // >:)
        'inner: for _ in 0..16 {
            let bogo = d + vec2(mq::gen_range(-1.0, 1.0), mq::gen_range(-1.0, 1.0));

            let bogodir  = vec2_to_dir(bogo);
            let bogodiri = dir_to_ivec2(&bogodir);

            let next_x = pos_x as i32 + bogodiri.x;
            let next_y = pos_y as i32 + bogodiri.y;

            // stupid checks for impossible cases (if "wall collisions" above somehow doesn't work lol)
            if (0 <= next_x && next_x <= 4).not() { continue; }
            if (0 <= next_y && next_y <= 4).not() { continue; }

            // blocked!
            if obstacles[next_y as usize][next_x as usize] == 1 {  continue; }

            // already visited. TRY AGAIN LOL!
            if visited[next_y as usize][next_x as usize] == 1 { continue; }

            // all checks pass
            out.push(bogodir);
            dirs[pos_y][pos_x] = bogodir;
            prev_dir = bogodiri.as_vec2();
            pos_x = next_x as usize;
            pos_y = next_y as usize;
            success = true;

            break 'inner;
        }

        if success.not() { return false; } // trapped somewhere lol

        if     pos_x == end.x as usize
            && pos_y == end.y as usize {
            // yay success here
            break;
        }

        // this can't happen
        if steps == 5*5 { return false; }

        steps += 1;
    }

    println!("\n{:?}", visited[0]);
    println!("{:?}", visited[1]);
    println!("{:?}", visited[2]);
    println!("{:?}", visited[3]);
    println!("{:?}", visited[4]);

    return true;
}

pub fn vec2_to_dir(d: Vec2) -> Dir {
    if       d.x > d.y.abs() { Dir::Right }
    else if  d.y > d.x.abs() { Dir::Down }
    else if -d.x > d.y.abs() { Dir::Left }
    else                     { Dir::Up }
}

pub fn rot_cw_90(a: Vec2) -> Vec2 {
    vec2(-a.y, a.x)
}

pub fn rot_ccw_90(a: Vec2) -> Vec2 {
    vec2(a.y, -a.x)
}

pub fn flip_x(a: (Vec2, Vec2)) -> (Vec2, Vec2) {
    (vec2(a.1.x, a.0.y), vec2(a.0.x, a.1.y))
}

pub fn flip_y(a: (Vec2, Vec2)) -> (Vec2, Vec2) {
    (vec2(a.0.x, a.1.y), vec2(a.1.x, a.0.y))
}

pub fn dir_to_mat2(dir: &Dir) -> Mat2 {
    match dir {
        Dir::Right => Mat2::from_cols(vec2( 1.0,  0.0), vec2( 0.0,  1.0)),
        Dir::Down  => Mat2::from_cols(vec2( 0.0,  1.0), vec2(-1.0,  0.0)),
        Dir::Left  => Mat2::from_cols(vec2(-1.0,  0.0), vec2( 0.0, -1.0)),
        Dir::Up    => Mat2::from_cols(vec2( 0.0, -1.0), vec2( 1.0,  0.0))
    }
}

pub fn dir_to_ivec2(dir: &Dir) -> IVec2 {
    match dir {
        Dir::Right => ivec2( 1,  0),
        Dir::Down  => ivec2( 0,  1),
        Dir::Left  => ivec2(-1,  0),
        Dir::Up    => ivec2( 0, -1),
    }
}

pub fn dir_to_vec2(dir: &Dir) -> Vec2 {
    match dir {
        Dir::Right => vec2( 1.0,  0.0),
        Dir::Down  => vec2( 0.0,  1.0),
        Dir::Left  => vec2(-1.0,  0.0),
        Dir::Up    => vec2( 0.0, -1.0),
    }
}

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
    fn test_vec2_to_dir() {
        assert!(matches!(vec2_to_dir(vec2(36.0, 1.0)), Dir::Right));
        assert!(matches!(vec2_to_dir(vec2(36.0, 35.0)), Dir::Right));
        assert!(matches!(vec2_to_dir(vec2(36.0, -35.0)), Dir::Right));
        assert!(matches!(vec2_to_dir(vec2(7.0, 35.0)), Dir::Down));
        assert!(matches!(vec2_to_dir(vec2(-69.0, 68.0)), Dir::Left));
        assert!(matches!(vec2_to_dir(vec2(70.0, -71.0)), Dir::Up));
        assert!(matches!(vec2_to_dir(vec2(-70.0, -71.0)), Dir::Up));
        assert!(matches!(vec2_to_dir(vec2(0.0, -1.0)), Dir::Up));
    }

    #[test]
    fn test_line_intersect() {
        assert!(line_segment_vs_line_intersect((vec2(5.0, 5.0), vec2(10.0, 10.0)), vec2(6.0, 0.0), vec2(0.0, 1.0)));

        assert!(line_segment_vs_line_intersect((vec2(5.0, 5.0), vec2(10.0, 10.0)), vec2(11.0, 0.0), vec2(0.0, 1.0)).not());

        assert!(line_segment_vs_line_intersect((vec2(5.0, 5.0), vec2(10.0, 10.0)), vec2(4.0, 0.0), vec2(0.0, 1.0)).not());
    }
}

