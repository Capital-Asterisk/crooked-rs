use crate::game::*;
use glam::UVec2;

lgrn::id_type!(ItemTypeId);
lgrn::id_type!(FeralItemId);

pub const ITEM_DEAD_DRONE   : ItemTypeId = ItemTypeId(0);
pub const ITEM_SCRAP        : ItemTypeId = ItemTypeId(1);
pub const ITEM_BATTERY      : ItemTypeId = ItemTypeId(2);
pub const ITEM_ALIGNITE     : ItemTypeId = ItemTypeId(3);
pub const ITEM_GUNPOWDER    : ItemTypeId = ItemTypeId(4);
pub const ITEM_BULLET       : ItemTypeId = ItemTypeId(5);
pub const ITEM_CLUMP        : ItemTypeId = ItemTypeId(6);
pub const ITEM_OBFUSCATOR   : ItemTypeId = ItemTypeId(7);


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

pub fn place_item(item_types: &Vec<ItemType>, feral_ids: &mut lgrn::IdReg<FeralItemId>, feral_data: &mut Vec<FeralItem>, feral_by_tile: &mut BTreeMap<(u8, u8), FeralItemId>, pos: UVec2, slot: ItemSlot) -> Result<FeralItemId, (FeralItemId, ItemSlot, bool)> {
    use std::collections::btree_map::Entry;
    match feral_by_tile.entry((pos.x as u8, pos.y as u8)) {
        Entry::Occupied(gwah) => {
            let feral: FeralItemId = *gwah.get();
            let d: &mut FeralItem = &mut feral_data[feral.0];
            let itype = &item_types[slot.itemtype.0];

            let mut transfered = false;
            let mut slotopt = Some(slot); // might be moved, might not

            // distribute
            for exslotasdf in &mut d.slots {
                if let Some(exslot) = exslotasdf {
                    if exslot.itemtype != slotopt.as_ref().unwrap().itemtype {
                        continue;
                    };
                    let transfer = u32::min(itype.stackable.saturating_sub(exslot.count),
                                            slotopt.as_ref().unwrap().count);

                    transfered |= transfer != 0;

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
                assert!(a.count != 0);
                return Err((feral, a, transfered));
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



pub fn take_items(slots: &mut Burger, itemtype: ItemTypeId, amount: u32) -> u32 {

    let mut remaining = amount;

    // check required items
    for draft_slot_opt in slots.iter_mut() {
        if let Some(draft_slot) = draft_slot_opt {

            if draft_slot.itemtype == itemtype {
                let consume = u32::min(draft_slot.count, remaining);
                remaining        -= consume;
                draft_slot.count -= consume;

                if draft_slot.count == 0 {
                    *draft_slot_opt = None;
                }
            }
        }
    }

    // fucklection-sort to move Somes left and Nones right
    let mut cake = slots.as_mut_slice();
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

    amount - remaining
}

pub fn feral_remove_if_empty(feral_ids: &mut lgrn::IdReg<FeralItemId>, feral_data: &mut Vec<FeralItem>, feral_by_tile: &mut BTreeMap<(u8, u8), FeralItemId>, feral: FeralItemId) {
    let d = &mut feral_data[feral.0];
    if d.slots.iter().all(|x| x.is_none()) {
        // No more slots left. feral item is gone 🦀
        feral_ids.remove(feral);
        feral_by_tile.remove(&(d.pos.x as u8, d.pos.y as u8));
    }
}

pub fn craft_item_recipe(item_types: &Vec<ItemType>, slots: &Burger, recipe: u32) -> Option<Burger> {
    match recipe {
        0 => craftt(item_types, slots,
            &mut [ItemSlot{itemtype: ITEM_DEAD_DRONE, count: 1}],
            &mut [ItemSlot{itemtype: ITEM_SCRAP, count: 1}, ItemSlot{itemtype: ITEM_BATTERY, count: 1}]),
        1 => craftt(item_types, slots,
            &mut [ItemSlot{itemtype: ITEM_SCRAP, count: 1}, ItemSlot{itemtype: ITEM_GUNPOWDER, count: 1}],
            &mut [ItemSlot{itemtype: ITEM_BULLET, count: 6}]),
        2 => craftt(item_types, slots,
            &mut [ItemSlot{itemtype: ITEM_ALIGNITE, count: 4}],
            &mut [ItemSlot{itemtype: ITEM_CLUMP, count: 1}]),
        69 => craftt(item_types, slots, &mut [ItemSlot{itemtype: ITEM_OBFUSCATOR, count: 1}], &mut []),
        _ => panic!()
    }
}

pub fn craft_machine_recipe(item_types: &Vec<ItemType>, slots: &Burger, recipe: u32) -> Option<Burger> {
    match recipe {
        5 => craftt(item_types, slots, &mut [ItemSlot{itemtype: ITEM_SCRAP, count: 2}], &mut []),
        6 => craftt(item_types, slots, &mut [ItemSlot{itemtype: ITEM_SCRAP, count: 4}, ItemSlot{itemtype: ITEM_BATTERY, count: 4}], &mut []),
        7 => craftt(item_types, slots, &mut [ItemSlot{itemtype: ITEM_SCRAP, count: 4}, ItemSlot{itemtype: ITEM_BATTERY, count: 4}, ItemSlot{itemtype: ITEM_ALIGNITE, count: 1}], &mut []),
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

    return Some(draft);
}

pub fn slots_contains(slots: &[Option<ItemSlot>], itype: ItemTypeId, mut amount: u32) -> bool {
    for exslotasdf in slots {
        if let Some(exslot) = exslotasdf {
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


pub fn slots_can_hold(item_types: &Vec<ItemType>, slots: &[Option<ItemSlot>], can_hold: &mut [(ItemTypeId, u32)]) -> bool {
    for exslotopt in slots {
        if let Some(exslot) = exslotopt {
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

