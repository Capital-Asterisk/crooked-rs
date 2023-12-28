
use glam::{vec2, Vec2, UVec2};
use std::{collections::BTreeMap, default};
use crate::lgrn;

pub const TILE_SIZE: Vec2 = vec2(64.0, 64.0);

lgrn::id_type!(ItemTypeId);
lgrn::id_type!(FeralItemId);
lgrn::id_type!(DroneId);
lgrn::id_type!(BulletId);
lgrn::id_type!(MachineId);


#[derive(Default)]
pub struct GameMain {
    pub world_size:     UVec2,
    pub player_pos:     Vec2,
    pub player_facing:  i8,
    pub hop_count:      u64,

    pub rail:           Vec<Rail>,

    pub drone_ids:      lgrn::IdReg<DroneId>,
    pub drone_pos:      Vec<Vec2>,
    pub drone_data:     Vec<Drone>,
    pub drone_by_x:     Vec<(DroneId, f32)>,

    pub bullet_ids:     lgrn::IdReg<BulletId>,
    pub bullet_pos:     Vec<Vec2>,
    pub bullet_data:    Vec<Bullet>,

    pub remove_drones:  Vec<DroneId>,
    pub remove_bullets: Vec<BulletId>,

    pub itemtype_data:  Vec<ItemType>,

    pub feral_ids:      lgrn::IdReg<FeralItemId>,
    pub feral_data:     Vec<FeralItem>,
    pub feral_by_tile:  BTreeMap<(u8, u8), FeralItemId>,

    pub tool:           ToolMode
}

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

pub enum Dir { Right, Down, Left, Up }
pub enum Bend { Forward, Right, Left }

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

#[derive(Clone, Default)]
pub struct FeralItem {
    pub pos: UVec2,
    pub slots: [Option<(ItemSlot, Vec2)>; 4]
}

#[derive(Clone)]
pub struct ItemSlot {
    pub itemtype: ItemTypeId,
    pub count: u32,
}

pub fn place_item(item_types: &Vec<ItemType>, feral_ids: &mut lgrn::IdReg<FeralItemId>, feral_data: &mut Vec<FeralItem>, feral_by_tile: &mut BTreeMap<(u8, u8), FeralItemId>, pos: UVec2, slot: ItemSlot) -> Result<FeralItemId, (FeralItemId, ItemSlot)> {
    use std::collections::btree_map::Entry;
    match feral_by_tile.entry((pos.x as u8, pos.y as u8)) {
        Entry::Occupied(gwah) => {
            let feral: FeralItemId = *gwah.get();
            let d: &mut FeralItem = &mut feral_data[feral.0];
            let itype = &item_types[slot.itemtype.0];

            let mut slotopt = Some(slot); // might be moved, might not

            // distribute
            for exslotasdf in &mut d.slots {
                if let Some((exslot, _)) = exslotasdf {
                    if exslot.itemtype != slotopt.as_ref().unwrap().itemtype {
                        continue;
                    };
                    let transfer = u32::min(itype.stackable.saturating_sub(exslot.count),
                                            slotopt.as_ref().unwrap().count);

                    exslot.count += transfer;
                    slotopt.as_mut().unwrap().count -= transfer;

                    if slotopt.as_ref().unwrap().count == 0 {
                        slotopt = None;
                        break;
                    }
                } else if let None = exslotasdf {
                    *exslotasdf = Some((slotopt.take().unwrap(), vec2(0.0, 0.0)));
                    break;
                }
            }

            if let Some(a) = slotopt {
                println!("nope.avi");
                assert!(a.count != 0);
                return Err((feral, a));
            }

            return Ok(feral);
        },
        Entry::Vacant(gwah) => {
            let feral: FeralItemId = *gwah.insert(feral_ids.create_resize());
            feral_data.resize(feral_ids.capacity(), Default::default());
            let d: &mut FeralItem = &mut feral_data[feral.0];
            d.pos = pos;
            d.slots[0] = Some((slot, vec2(0.0, 0.0)));

            return Ok(feral);
        }
    };
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

