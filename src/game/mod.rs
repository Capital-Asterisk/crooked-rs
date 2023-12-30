
use glam::{vec2, Vec2, UVec2};
use std::{collections::BTreeMap, default};
use crate::lgrn;

use std::ops::Not;

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

    // [0] is drawn as the front since items are taken from [0]. New items are pushed into the back
    // to behave as a FIFO
    pub slots: Burger
}

#[derive(Clone, Default)]
pub struct ItemSlot {
    pub itemtype: ItemTypeId,
    pub count: u32,
}

type Burger = [Option<ItemSlot>; 4];

pub fn slots_contains(slots: &[Option<(ItemSlot, Vec2)>], itype: ItemTypeId, mut amount: u32) -> bool {
    for exslotasdf in slots {
        if let Some((exslot, _)) = exslotasdf {
            if exslot.itemtype != itype {
                continue;
            };
            amount = amount.saturating_sub(exslot.count);
            if amount == 0 {
                return true;
            }
        }
    }
    return false;
}

pub fn slots_can_hold(item_types: &Vec<ItemType>, slots: &[Option<(ItemSlot, Vec2)>], can_hold: &mut [(ItemTypeId, u32)]) -> bool {
    for exslotopt in slots {
        if let Some((exslot, _)) = exslotopt {
            if let Some((ref mut itypeid, ref mut remaining)) = can_hold.iter_mut().find(|(ref itypeid, _)| *itypeid == exslot.itemtype) {
                let itype = &item_types[itypeid.0];
                *remaining -= u32::min(itype.stackable.saturating_sub(exslot.count), *remaining);
                if *remaining == 0 {
                    *itypeid = Default::default();
                }
            }
        } else {
            if let Some((ref mut itypeid, ref mut remaining)) = can_hold.iter_mut().find(|(_, remaining)| *remaining != 0) {
                *remaining = 0; // empty slot. put anything here
                *itypeid = Default::default();
            }
        }
    }
    return can_hold.iter().all(|(_, remaining)| *remaining == 0);
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
                if let Some(exslot) = exslotasdf {
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
                    *exslotasdf = Some(slotopt.take().unwrap());
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
            d.slots[0] = Some(slot);

            return Ok(feral);
        }
    };
}

pub fn feral_remove_if_empty(feral_ids: &mut lgrn::IdReg<FeralItemId>, feral_data: &mut Vec<FeralItem>, feral_by_tile: &mut BTreeMap<(u8, u8), FeralItemId>, feral: FeralItemId) {
    let d = &mut feral_data[feral.0];
    if d.slots.iter().all(|x| x.is_none()) {
        // No more slots left. feral item is gone 🦀
        feral_ids.remove(feral);
        feral_by_tile.remove(&(d.pos.x as u8, d.pos.y as u8));
    }
}

pub const ITEM_DEAD_DRONE   : ItemTypeId = ItemTypeId(0);
pub const ITEM_SCRAP        : ItemTypeId = ItemTypeId(1);
pub const ITEM_BATTERY      : ItemTypeId = ItemTypeId(2);
pub const ITEM_ALIGNITE     : ItemTypeId = ItemTypeId(3);
pub const ITEM_GUNPOWDER    : ItemTypeId = ItemTypeId(4);

pub fn craft_item_recipe(item_types: &Vec<ItemType>, slots: &Burger, recipe: u32) -> Option<Burger> {
    match recipe {
        0 => craftt(item_types, slots,
            &mut [ItemSlot{itemtype: ITEM_DEAD_DRONE, count: 1}],
            &mut [ItemSlot{itemtype: ITEM_SCRAP, count: 1}, ItemSlot{itemtype: ITEM_BATTERY, count: 1}]),

        _ => panic!()
    }
}

pub fn craft_machine_recipe(item_types: &Vec<ItemType>, slots: &Burger, recipe: u32) -> Option<Burger> {
    match recipe {
        5 => craftt(item_types, slots, &mut [ItemSlot{itemtype: ITEM_SCRAP, count: 2}], &mut []),
        _ => panic!()
    }
}



pub fn craftt(item_types: &Vec<ItemType>, slots: &Burger, req: &mut [ItemSlot], out: &mut [ItemSlot]) -> Option<Burger> {

    let mut draft: Burger = slots.clone();

    // check required items
    for draft_slot_opt in draft.iter_mut() {
        if let Some(draft_slot) = draft_slot_opt {
            if let Some(req_slot) = req.iter_mut().find(|req_slot_n| draft_slot.itemtype == req_slot_n.itemtype) {
                let consume = u32::min(draft_slot.count, req_slot.count);

                req_slot  .count -= consume;
                draft_slot.count -= consume;

                if draft_slot.count == 0 {
                    *draft_slot_opt = None;
                }
            }
        }
    }

    if req.iter().any(|req_slot_n| req_slot_n.count != 0) {
        return None; // Requirements not satisfied
    }

    // fucklection-sort to move Somes left and Nones right
    let mut cake = draft.as_mut_slice();
    while cake.is_empty().not() {
        if cake[0].is_none() {
            if let Some(idk) = cake.iter_mut().find(|foo| foo.is_some()) {
                cake[0] = idk.take();
            } else {
                break; // no more Some left
            }
        } else {
            cake = &mut cake[1..];
        }
    }

    // dump output items into slot
    'outer: for out_slot in out.iter_mut() {
        for draft_slot_opt in draft.iter_mut() {
            if out_slot.count == 0 {
                continue 'outer;
            }

            if let Some(draft_slot) = draft_slot_opt {
                if draft_slot.itemtype == out_slot.itemtype {
                    let stackable = &item_types[draft_slot.itemtype.0].stackable;
                    let transfer = u32::min(stackable - draft_slot.count, out_slot.count);

                    out_slot  .count -= transfer;
                    draft_slot.count += transfer;
                }
            } else {
                // empty slot. put anything here
                *draft_slot_opt = Some(out_slot.clone());
                *out_slot = Default::default();
            }
        }

        if out_slot.count != 0 {
            return None;
        }
    }

/*

    for draft_slot_opt in draft.iter_mut() {
        if let Some(draft_slot) = draft_slot_opt {
            if let Some(out_slot) = out.iter_mut().find(|out_slot_n| draft_slot.itemtype == out_slot_n.itemtype) {

            }
        } else {
            // empty slot. put anything here
            if let Some(out_slot) = out.iter_mut().find(|out_slot_n| out_slot_n.count != 0) {
                *draft_slot_opt = Some(out_slot.clone());
                *out_slot = Default::default();
            }
        }
    }*/

    return Some(draft);
}


/*
pub fn slots_recipies(slots: &[Option<(ItemSlot, Vec2)>], itype: ItemTypeId, mut amount: u32) -> [bool; 3] {

}*/



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

