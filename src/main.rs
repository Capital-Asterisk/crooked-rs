use std::cmp::Ordering;
use std::ops::Not;
use std::time::{SystemTime, UNIX_EPOCH};

use glam::Mat2;
use obfuscation::draw::TileThing;
use obfuscation::game::GameMain;
use obfuscation::game::*;
use obfuscation::draw;

use glam::{Vec2, vec2, uvec2, ivec2};

pub extern crate glam;


pub mod mq
{
    pub use macroquad::prelude::*;
    pub use macroquad::audio::*;
    pub use macroquad::rand::*;
}



fn repitch(data: &mut [u8], sample_rate_original: u32, rate_shift: f32)
{
    let sample_rate         = ((sample_rate_original as f32) * rate_shift) as u32;
    let sample_rate_bytes   = sample_rate    .to_le_bytes();
    let data_rate_bytes     = (sample_rate*2).to_le_bytes();

    data[24] = sample_rate_bytes[0];
    data[25] = sample_rate_bytes[1];
    data[26] = sample_rate_bytes[2];
    data[27] = sample_rate_bytes[3];
    data[28] = data_rate_bytes[0];
    data[29] = data_rate_bytes[1];
    data[30] = data_rate_bytes[2];
    data[31] = data_rate_bytes[3];
}

#[macroquad::main("obfuscated bird thing")]
async fn main() {

    //mq::srand(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

    let mut data = mq::load_file("tf/custom/step.wav").await.unwrap();

    let semitone = 2.0_f32.powf(1.0/12.0);
    let mut step_sounds: Vec<mq::Sound> = Vec::with_capacity(4);
    step_sounds.push(mq::load_sound_from_bytes(&data).await.unwrap());
    repitch(&mut data, 44100, 1.0 * semitone.powi(4));
    step_sounds.push(mq::load_sound_from_bytes(&data).await.unwrap());
    repitch(&mut data, 44100, 1.1 * semitone.powi(7));
    step_sounds.push(mq::load_sound_from_bytes(&data).await.unwrap());
    repitch(&mut data, 44100, 0.9 * semitone.powi(10));
    step_sounds.push(mq::load_sound_from_bytes(&data).await.unwrap());

    let changedir_sound = mq::load_sound("tf/custom/changedir.wav").await.unwrap();

    let shoot0_sound = mq::load_sound("tf/custom/shoot0.wav").await.unwrap();
    let shoot1_sound = mq::load_sound("tf/custom/shoot1.wav").await.unwrap();
    let pickup_sound = mq::load_sound("tf/custom/pickup.wav").await.unwrap();
    let place_sound = mq::load_sound("tf/custom/place.wav").await.unwrap();
    let deflect_sound = mq::load_sound("tf/custom/deflect.wav").await.unwrap();
    let explode_sound = mq::load_sound("tf/custom/explode.wav").await.unwrap();
    let craft_sound = mq::load_sound("tf/custom/craft.wav").await.unwrap();
    let reload_sound = mq::load_sound("tf/custom/reload.wav").await.unwrap();
    let press_sound = mq::load_sound("tf/custom/press.wav").await.unwrap();
    let obfuscator_sound = mq::load_sound("tf/custom/obfuscator.wav").await.unwrap();

    //let step: mq::Sound;
    //step = mq::load_sound_from_bytes(&data).await.unwrap();

    let mut controls = Controls
    {
        walk: vec2(0.0, 0.0)
    };

    let mut game_main: GameMain = Default::default();

    game_main.world_size = uvec2(80, 25);

    for x in 0..game_main.world_size.x {
        game_main.rail.push(Rail { pos: uvec2(x, game_main.world_size.y/2), dir: Dir::Right, bend: Bend::Forward});
    }
    // game_main.rail.push(Rail { pos: uvec2(10, 4), dir: Dir::Right, bend: Bend::Right});
    // game_main.rail.push(Rail { pos: uvec2(10, 5), dir: Dir::Down,  bend: Bend::Forward});
    // game_main.rail.push(Rail { pos: uvec2(10, 6), dir: Dir::Down,  bend: Bend::Forward});
    // game_main.rail.push(Rail { pos: uvec2(10, 7), dir: Dir::Down,  bend: Bend::Forward});
    // game_main.rail.push(Rail { pos: uvec2(10, 8), dir: Dir::Down,  bend: Bend::Right});
    // game_main.rail.push(Rail { pos: uvec2(9,  8), dir: Dir::Left,  bend: Bend::Forward});
    // game_main.rail.push(Rail { pos: uvec2(8,  8), dir: Dir::Left,  bend: Bend::Right});
    // game_main.rail.push(Rail { pos: uvec2(8,  7), dir: Dir::Up,    bend: Bend::Left});
    // game_main.rail.push(Rail { pos: uvec2(7,  7), dir: Dir::Left,  bend: Bend::Left});
    // game_main.rail.push(Rail { pos: uvec2(7,  8), dir: Dir::Down,  bend: Bend::Forward});
    // for y in 9..48 {
    //     game_main.rail.push(Rail { pos: uvec2(7,  y), dir: Dir::Down, bend: Bend::Forward});
    // }

    game_main.drone_per_second = 5.0;

    regen_rail_by_tile(&game_main.rail, &mut game_main.rail_by_tile);

    game_main.drone_ids.resize(512);
    game_main.drone_pos.resize(512, vec2(0.0, 0.0));
    game_main.drone_data.resize(512, Drone{rail_idx: 0, rail_pos: 0.0, speed: 0.0});
    game_main.drone_by_x.reserve(512);

    game_main.bullet_ids.resize(128);
    game_main.bullet_pos.resize(512, vec2(0.0, 0.0));
    game_main.bullet_data.resize(512, Bullet{dir: vec2(0.0, 0.0), speed: 0.0, travel: 0.0, travel_max: 0.0});

    game_main.itemtype_data.push(ItemType {
        sprite: draw::sprite(0, 3),
        stackable: 1,
        name: "Destroyed Logistics Drone",
        desc: "gwah"
    });
    game_main.itemtype_data.push(ItemType {
        sprite: draw::sprite(1, 3),
        stackable: 69,
        name: "Scrap Metal",
        desc: "Level 1 craft item"
    });
    game_main.itemtype_data.push(ItemType {
        sprite: draw::sprite(2, 3),
        stackable: 69,
        name: "Battery",
        desc: "Energy-dense solid"
    });
    game_main.itemtype_data.push(ItemType {
        sprite: draw::sprite(3, 3),
        stackable: 69,
        name: "Red Alignite crystal",
        desc: "Extremely cubic, rotation-locked with the planet"
    });
    game_main.itemtype_data.push(ItemType {
        sprite: draw::sprite(4, 3),
        stackable: 69,
        name: "Gunpowder",
        desc: "Energy-dense explosive solid"
    });
    game_main.itemtype_data.push(ItemType {
        sprite: draw::sprite(5, 3),
        stackable: 69,
        name: "Bullet",
        desc: ""
    });
    game_main.itemtype_data.push(ItemType {
        sprite: draw::sprite(6, 3),
        stackable: 1,
        name: "Red Alignite clump",
        desc: ""
    });
    game_main.itemtype_data.push(ItemType {
        sprite: draw::sprite(6, 4),
        stackable: 1,
        name: "Obfuscator Charge",
        desc: ""
    });


    game_main.feral_ids.resize(32);
    game_main.mach_ids.resize(32);

    game_main.player_pos = vec2(40.0 * TILE_SIZE.x, 10.0 * TILE_SIZE.y);

    // place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(3, 4), ItemSlot { itemtype: ItemTypeId(2), count: 44 }).ok();
    // place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(3, 4), ItemSlot { itemtype: ItemTypeId(2), count: 44 }).ok();
    // place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(3, 4), ItemSlot { itemtype: ItemTypeId(2), count: 44 }).ok();
    // place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(3, 4), ItemSlot { itemtype: ItemTypeId(2), count: 44 }).ok();
    // place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(2, 2), ItemSlot { itemtype: ItemTypeId(0), count: 55 }).ok();

    for _ in 0..24 {
        let pos = uvec2(mq::gen_range(0, game_main.world_size.x), mq::gen_range(0, game_main.world_size.y));
        let amount = mq::gen_range(10, 60);
        place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, pos, ItemSlot { itemtype: ITEM_BULLET, count: amount }).ok();
    }

    //place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(2, 3), ItemSlot { itemtype: ITEM_DEAD_DRONE, count: 54 }).ok();
    place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(40, 5), ItemSlot { itemtype: ITEM_OBFUSCATOR, count: 1 }).ok();
    place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(42, 6), ItemSlot { itemtype: ITEM_OBFUSCATOR, count: 1 }).ok();
    place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(36, 9), ItemSlot { itemtype: ITEM_OBFUSCATOR, count: 1 }).ok();

    let mut game_draw: draw::GameDraw = draw::make_game_draw().await;

    //let mut frame_count: u64 = 0;

    loop {

        let delta: f32 = mq::get_frame_time().min(0.1);

        controls.walk.x = (mq::is_key_down(mq::KeyCode::D) as i32 - mq::is_key_down(mq::KeyCode::A) as i32) as f32;
        controls.walk.y = (mq::is_key_down(mq::KeyCode::S) as i32 - mq::is_key_down(mq::KeyCode::W) as i32) as f32;
        let is_walking = controls.walk.length_squared() > 0.01;

        game_draw.cam_size = if mq::is_key_down(mq::KeyCode::Z) {25.0} else {10.0};

        draw::draw_game(&game_main, &mut game_draw);
        game_draw.clock_1s = (game_draw.clock_1s + delta).fract();

        // Walk
        game_draw.player_hop_time += delta;
        if is_walking && game_draw.player_hop_time > game_draw.player_hop_rate {
            game_main.hop_count += 1;
            game_draw.player_hop_time = 0.0;
            mq::play_sound_once(&step_sounds[mq::gen_range(0, step_sounds.len())]);
        }
        if controls.walk.x.abs() > 0.01 {
            game_main.player_facing = controls.walk.x.signum() as i8;
        }
        game_main.player_pos += controls.walk * delta * TILE_SIZE.x * 5.0;
        game_main.player_pos = game_main.player_pos.clamp(vec2(0.0, 0.0), game_main.world_size.as_vec2() * TILE_SIZE);

        // cycle tool
        if mq::is_key_pressed(mq::KeyCode::Q) {

            match &game_main.tool {
                ToolMode::Construct(drag) => {
                    if let Drag::None = drag {
                        game_main.tool = ToolMode::GunPod;
                    }
                },
                ToolMode::GunPod => { game_main.tool = ToolMode::Construct(Drag::None) },
            }
        }

        match &mut game_main.tool {
            ToolMode::Construct(drag) => {
                // Pick up and place items and machines
                if mq::is_mouse_button_pressed(mq::MouseButton::Left)
                || mq::is_key_pressed(mq::KeyCode::E) {
                    if let Drag::None = drag {
                        match game_draw.under_cursor {
                            TileThing::Feral(feral) => {
                                let d = &mut game_main.feral_data[feral.0];

                                let slot = &mut d.slots[0]; //&mut d.slots.iter_mut().find(|s| s.is_some()).unwrap();

                                let slot_extracted = slot.take().unwrap();
                                d.slots.rotate_left(1);

                                feral_remove_if_empty(&mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, feral);

                                *drag = Drag::Item(slot_extracted);

                                mq::play_sound(&pickup_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });
                            },
                            TileThing::Machine(mach) => {
                                let d = &mut game_main.mach_data[mach.0];
                                d.pos = None;
                                *drag = Drag::Machine(mach);
                                game_main.mach_by_tile.remove(&(game_draw.mouse_select.x as u8, game_draw.mouse_select.y as u8));
                                mq::play_sound(&pickup_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });
                            },
                            _ => {}
                        };
                    } else if let Drag::Item(slot) = drag {

                        if matches!(game_draw.under_cursor, TileThing::Machine(_)).not() {

                            let Drag::Item(slot) = std::mem::take(drag) else { panic!() };

                            match place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, game_draw.mouse_select, slot) {
                                Ok(_) => {
                                    mq::play_sound(&place_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });
                                },
                                Err((_, slot, transfered)) => {
                                    *drag = Drag::Item(slot);
                                    if transfered {
                                        mq::play_sound(&place_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });
                                    }
                                }
                            }
                        } else if let TileThing::Machine(mach) = game_draw.under_cursor {
                            // Reload turrets
                            if slot.itemtype == ITEM_BULLET {
                                let d = &mut game_main.mach_data[mach.0];
                                if let MachineSpec::Turret { ammo, can_fire_time_us: _ } = &mut d.spec {
                                    let transfer = u32::min(69_u32.saturating_sub(*ammo), slot.count);
                                    *ammo += transfer;
                                    slot.count -= transfer;

                                    if slot.count == 0 {
                                        *drag = Drag::None;
                                    }
                                    mq::play_sound(&reload_sound, mq::PlaySoundParams { looped: false, volume: 0.8 });
                                };
                            }
                        }
                    } else if let Drag::Machine(mach) = drag {

                        let valid_placement = matches!(game_draw.under_cursor, TileThing::None) && game_main.rail.iter().all(|x| x.pos != game_draw.mouse_select);

                        if valid_placement {

                            mq::play_sound(&place_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });

                            game_main.mach_by_tile.insert((game_draw.mouse_select.x as u8, game_draw.mouse_select.y as u8), mach.clone());

                            game_main.mach_data[mach.0].pos = Some(game_draw.mouse_select);

                            *drag = Drag::None;

                        }
                    }
                }

                // Rotate machine
                if mq::is_key_pressed(mq::KeyCode::R) {
                    if let Drag::Machine(mach) = drag {
                        let d = &mut game_main.mach_data[mach.0];
                        d.dir = match d.dir {
                            Dir::Right  => Dir::Up,
                            Dir::Up     => Dir::Left,
                            Dir::Left   => Dir::Down,
                            Dir::Down   => Dir::Right,
                        };
                    }
                }

                // Craft
                if let Drag::None = drag {
                    if game_main.rail_by_tile.contains_key(&(game_draw.mouse_select.x as u8, game_draw.mouse_select.y as u8)).not() {
                        if mq::is_key_pressed(mq::KeyCode::Key1) { // disassemble
                            if let TileThing::Feral(feral) = game_draw.under_cursor {
                                if let Some(gwah) = craft_item_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 0) {
                                    game_main.feral_data[feral.0].slots = gwah;
                                    mq::play_sound(&craft_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });
                                }
                            }
                        } else if mq::is_key_pressed(mq::KeyCode::Key2) { // bullet
                            if let TileThing::Feral(feral) = game_draw.under_cursor {
                                if let Some(gwah) = craft_item_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 1) {
                                    game_main.feral_data[feral.0].slots = gwah;
                                    mq::play_sound(&craft_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });
                                }
                            }
                        } else if mq::is_key_pressed(mq::KeyCode::Key3) { // clump
                            if let TileThing::Feral(feral) = game_draw.under_cursor {
                                if let Some(gwah) = craft_item_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 2) {
                                    game_main.feral_data[feral.0].slots = gwah;
                                    mq::play_sound(&craft_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });
                                }
                            }
                        } else if mq::is_key_pressed(mq::KeyCode::Key5) { // turret
                            if let TileThing::Feral(feral) = game_draw.under_cursor {
                                if let Some(gwah) = craft_machine_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 5) {
                                    game_main.feral_data[feral.0].slots = gwah;
                                    mq::play_sound(&craft_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });

                                    feral_remove_if_empty(&mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, feral);

                                    let mach = game_main.mach_ids.create_resize();
                                    game_main.mach_data.resize(game_main.mach_ids.capacity(), Default::default());
                                    game_main.mach_data[mach.0].spec = MachineSpec::Turret { ammo: 0, can_fire_time_us: game_main.time_us };
                                    *drag = Drag::Machine(mach);
                                }
                            }
                        } else if mq::is_key_pressed(mq::KeyCode::Key6) { // conveyor
                            if let TileThing::Feral(feral) = game_draw.under_cursor {
                                if let Some(gwah) = craft_machine_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 6) {
                                    game_main.feral_data[feral.0].slots = gwah;
                                    mq::play_sound(&craft_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });

                                    feral_remove_if_empty(&mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, feral);

                                    let mach = game_main.mach_ids.create_resize();
                                    game_main.mach_data.resize(game_main.mach_ids.capacity(), Default::default());

                                    game_main.mach_data[mach.0].spec = MachineSpec::Conveyor{item: Default::default(), filter: false, can_move_time_us: game_main.time_us, can_dump_time_us: game_main.time_us};
                                    *drag = Drag::Machine(mach);
                                }
                            }
                        } else if mq::is_key_pressed(mq::KeyCode::Key7) { // filterveyor
                            if let TileThing::Feral(feral) = game_draw.under_cursor {
                                if let Some(gwah) = craft_machine_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 7) {
                                    game_main.feral_data[feral.0].slots = gwah;
                                    mq::play_sound(&craft_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });

                                    feral_remove_if_empty(&mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, feral);

                                    let mach = game_main.mach_ids.create_resize();
                                    game_main.mach_data.resize(game_main.mach_ids.capacity(), Default::default());

                                    game_main.mach_data[mach.0].spec = MachineSpec::Conveyor{item: Default::default(), filter: true, can_move_time_us: game_main.time_us, can_dump_time_us: game_main.time_us};
                                    *drag = Drag::Machine(mach);
                                }
                            }
                        } else if mq::is_key_pressed(mq::KeyCode::R) {
                            if let TileThing::Feral(feral) = game_draw.under_cursor {
                                if let Some(gwah) = craft_item_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 69) {

                                    if world_obfuscate(game_draw.mouse_select, &mut game_main.rail, game_main.world_size) {
                                        game_main.feral_data[feral.0].slots = gwah;
                                        feral_remove_if_empty(&mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, feral);

                                        mq::play_sound(&obfuscator_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });
                                        regen_rail_by_tile(&game_main.rail, &mut game_main.rail_by_tile);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ToolMode::GunPod => {

                // Reload
                if mq::is_key_pressed(mq::KeyCode::R) {
                    if let TileThing::Feral(feral) = game_draw.under_cursor {

                        let got_bullets = take_items(&mut game_main.feral_data[feral.0].slots, ITEM_BULLET, PLAYER_GUN_AMMO_MAX - game_main.player_gun_ammo);
                        feral_remove_if_empty(&mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, feral);

                        if got_bullets != 0 {
                            game_main.player_gun_ammo += got_bullets;
                            mq::play_sound(&reload_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });
                        }
                    }
                }
            }
        }

        use Dir;
        use Bend;

        // Remove drones
        for drone in &game_main.remove_drones {
            game_main.drone_ids.remove(*drone);
        }
        game_main.remove_drones.clear();

        // Move drones
        for drone in game_main.drone_ids.iter_ids() {
            let d: &mut Drone     = &mut game_main.drone_data[drone.0];
            let p: &mut Vec2      = &mut game_main.drone_pos[drone.0];

            // hack: index goes out of bounds when rail gets shortened when using obfuscation charge
            if d.rail_idx >= game_main.rail.len() {
                d.rail_idx = game_main.rail.len() - 1;
            }

            let r: &Rail          = &game_main.rail[d.rail_idx];

            let mut dir = dir_to_vec2(&r.dir);

            if d.rail_pos > 0.5 {
                dir = match r.bend {
                    Bend::Forward => dir,
                    Bend::Right   => rot_cw_90(dir),
                    Bend::Left    => rot_ccw_90(dir)
                };
            }

            *p = (r.pos.as_vec2() + vec2(0.5, 0.5-0.125) + dir * (d.rail_pos - 0.5)) * TILE_SIZE;

            let rail_pos_next = d.rail_pos+d.speed*delta;
            let midway = d.rail_pos < 0.5 && 0.5 < rail_pos_next;

            if midway {
                if let Some(feral) = game_main.feral_by_tile.get(&(r.pos.x as u8, r.pos.y as u8)) {
                    let mut somethinghappen = false;
                    for slot_opt in &mut game_main.feral_data[feral.0].slots {
                        if let Some(slot) = slot_opt {
                            if slot.itemtype == ITEM_BATTERY {
                                somethinghappen = true;
                                slot.itemtype = ITEM_GUNPOWDER;
                            }
                        }
                    }
                    if somethinghappen {
                        //d.speed *= 2.0/16.0;
                        let vol = (1.0 - (game_draw.cam_center - *p).length()/12.0/TILE_SIZE.x).min(1.0);
                        if vol > 0.0 {
                            mq::play_sound(&press_sound, mq::PlaySoundParams { looped: false, volume: vol });
                        }
                    }
                }
            }

            if matches!(r.bend, Bend::Forward).not() && midway {
                d.rail_pos = 0.5;
                d.speed *= 15.0/16.0;

                let vol = (1.0 - (game_draw.cam_center - *p).length()/8.0/TILE_SIZE.x).min(1.0) * 0.2;
                if vol > 0.0 {
                    mq::play_sound(&changedir_sound, mq::PlaySoundParams { looped: false, volume: vol });
                }

            } else {
                d.rail_pos = rail_pos_next;
                d.speed = f32::min(d.speed + delta*0.2, 32.0);
            }

            if d.rail_pos > 1.0 {
                d.rail_pos -= 1.0;
                d.rail_idx += 1;

                if d.rail_idx == game_main.rail.len() {
                    game_main.remove_drones.push(drone);
                }
            }
        }


        if game_main.drone_timer < 0.0 {
            let id = game_main.drone_ids.create_resize();
            game_main.drone_pos.resize(game_main.drone_ids.capacity(), vec2(-10000.0, -100000.0));
            game_main.drone_data.resize(game_main.drone_ids.capacity(), Drone{rail_idx: 0, rail_pos: 0.0, speed: 0.0});

            game_main.drone_data[id.0] = Drone{rail_idx: 0, rail_pos: 0.0, speed: 32.0};

            let pos = &mut game_main.drone_pos[id.0];
            *pos = vec2(-10000.0, -100000.0);
            game_main.drone_by_x.push((id, pos.x));

            game_main.drone_timer += 1.0 / game_main.drone_per_second;
        }
        game_main.drone_timer -= delta;

        let mut new_len = game_main.drone_by_x.len();
        let mut slice = game_main.drone_by_x.as_mut_slice();
        while slice.is_empty().not() {
            let delete = game_main.drone_ids.exists(slice[0].0).not();
            if delete {
                // swap-and-pop (swap_remove) to delete
                let last_idx = slice.len()-1;
                new_len -= 1;
                slice[0] = slice[last_idx];
                slice = &mut slice[0..last_idx];
            } else {
                // update x
                slice[0].1 = game_main.drone_pos[slice[0].0.0].x;

                // Next element
                slice = &mut slice[1..];
            }
        }
        game_main.drone_by_x.truncate(new_len);

        game_main.drone_by_x.sort_unstable_by(|lhs, rhs| {
            lhs.1.partial_cmp(&rhs.1).unwrap()
        });



        // Delete bullets
        for id in &game_main.remove_bullets {
            game_main.bullet_ids.remove(*id);
        }
        game_main.remove_bullets.clear();

        // Move bullets
        for id in game_main.bullet_ids.iter_ids() {
            let d: &mut Bullet = &mut game_main.bullet_data[id.0];
            let p: &mut Vec2         = &mut game_main.bullet_pos[id.0];

            let trav = d.speed * delta;
            d.travel += trav;

            if d.travel > d.travel_max {
                game_main.remove_bullets.push(id);
            }

            // search for drones from this X value and above
            let first_x: f32 = p.x - 0.5*TILE_SIZE.x;

            // lower_bound: https://stackoverflow.com/questions/75790347/
            let mut idx = game_main.drone_by_x.binary_search_by(|x| match x.1.total_cmp(&first_x) {
                Ordering::Equal => Ordering::Greater,
                ord => ord,
            }).unwrap_err();

            //println!("wot: {} {}", idx, game_main.drone_by_x.len());


            while idx < game_main.drone_by_x.len() {
                let drone_id = game_main.drone_by_x[idx].0;
                let drone_pos = game_main.drone_pos[drone_id.0];
                let drone_tl = drone_pos - TILE_SIZE * 0.5;
                let drone_br = drone_pos + TILE_SIZE * 0.5;

                if p.x > drone_br.x {
                    break;
                }

                if    drone_tl.x < p.x && p.x < drone_br.x
                   && drone_tl.y < p.y && p.y < drone_br.y {

                    let drone_tr = vec2(drone_br.x, drone_tl.y);
                    let drone_bl = vec2(drone_tl.x, drone_br.y);

                    let norm = (|| {

                        if d.dir.x < 0.0 {
                            if line_segment_vs_line_intersect((drone_tr, drone_br), *p, d.dir) {
                                return Some(vec2(1.0, 0.0));
                            }
                        } else if 0.0 < d.dir.x {
                            if line_segment_vs_line_intersect((drone_tl, drone_bl), *p, d.dir) {
                                return Some(vec2(-1.0, 0.0));
                            }
                        }

                        if d.dir.y < 0.0 {
                            if line_segment_vs_line_intersect((drone_bl, drone_br), *p, d.dir) {
                                return Some(vec2(0.0, 1.0));
                            }
                        } else if 0.0 < d.dir.y {
                            if line_segment_vs_line_intersect((drone_tl, drone_tr), *p, d.dir) {
                                return Some(vec2(0.0, -1.0));
                            }
                        }

                        return None;
                    })();


                    if let Some(norm) = norm {

                        let dot = norm.dot(-d.dir);

                        let vol = (1.0 - (game_draw.cam_center - *p).length()/12.0/TILE_SIZE.x).min(1.0);

                        if dot > f32::cos(26.0_f32.to_radians()) {
                            game_main.remove_drones.push(drone_id);
                            game_main.remove_bullets.push(id);
                            place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, (drone_pos / TILE_SIZE).floor().as_uvec2(), ItemSlot { itemtype: ItemTypeId(0), count: 1 }).ok();

                            if vol > 0.0 {
                                mq::play_sound(&explode_sound, mq::PlaySoundParams { looped: false, volume: vol*0.8 });
                            }

                        } else {
                            d.dir = d.dir + 2.0*norm*dot;
                            mq::play_sound(&deflect_sound, mq::PlaySoundParams { looped: false, volume: vol });
                        }
                    }

                    break;
                }
                idx += 1;
            }

            *p += d.dir * trav;

        }


        const TURRET_PERIOD: u64 = 1500000u64; // 1.5 seconds
        const CONVEY_TAKE_PERIOD: u64 = 250000u64; // 0.25 seconds
        const CONVEY_DUMP_PERIOD: u64 = 100000u64; // 0.1 seconds


        for mach in game_main.mach_ids.iter_ids() {

            let pos_opt = game_main.mach_data[mach.0].pos.clone();


            if let Some(pos) = pos_opt {

                // note: borrow checker not happy when accessing multiple machines at a time

                // Turrets shoot
                if matches!(game_main.mach_data[mach.0].spec, MachineSpec::Turret { ammo: _, can_fire_time_us: _ })  {

                    let d = &mut game_main.mach_data[mach.0];
                    let MachineSpec::Turret { ammo, can_fire_time_us } = &mut d.spec else { panic!(); };

                    if (*ammo != 0) && (*can_fire_time_us < game_main.time_us) {

                        let ppos = (pos.as_vec2() + vec2(0.5, 0.5)) * TILE_SIZE;
                        let dirmat = dir_to_mat2(&d.dir);

                        let drone_detected: bool = (|| {

                            let point_a = ppos + dirmat.mul_vec2(vec2(0.0, 0.6)) * TILE_SIZE;
                            let point_b = ppos + dirmat.mul_vec2(vec2(5.0, -0.6)) * TILE_SIZE;
                            let tl = Vec2::min(point_a, point_b);
                            let br = Vec2::max(point_a, point_b);

                            // search for drones from this X value and above
                            let first_x: f32 = tl.x;

                            // lower_bound: https://stackoverflow.com/questions/75790347/
                            let mut idx = game_main.drone_by_x.binary_search_by(|x| match x.1.total_cmp(&first_x) {
                                Ordering::Equal => Ordering::Greater,
                                ord => ord,
                            }).unwrap_err();

                            while idx < game_main.drone_by_x.len() {
                                let drone_id = game_main.drone_by_x[idx].0;
                                let drone_pos = game_main.drone_pos[drone_id.0];

                                if br.x < drone_pos.x {
                                    break;
                                }

                                if    (tl.x < drone_pos.x && drone_pos.x < br.x)
                                   && (tl.y < drone_pos.y && drone_pos.y < br.y) {
                                    return true;
                                }

                                idx += 1;
                            }

                            return false;
                        })();

                        if drone_detected {
                            *ammo -= 1;
                            *can_fire_time_us = game_main.time_us + TURRET_PERIOD - (game_main.time_us - *can_fire_time_us)%TURRET_PERIOD;


                            let bullet = game_main.bullet_ids.create().unwrap();
                            game_main.bullet_pos[bullet.0]  = ppos + dirmat.x_axis * 0.5*TILE_SIZE;
                            game_main.bullet_data[bullet.0] = Bullet{dir: dirmat.x_axis, speed: 1200.0, travel: 0.0, travel_max: TILE_SIZE.x * 4.5};

                            let vol = (1.0 - (game_draw.cam_center - ppos).length()/12.0/TILE_SIZE.x).min(1.0);
                            if vol > 0.0 {
                                mq::play_sound(&shoot1_sound, mq::PlaySoundParams { looped: false, volume: vol });
                            }
                        }
                    }
                } else if matches!(game_main.mach_data[mach.0].spec, MachineSpec::Conveyor { item: _, filter: _, can_move_time_us: _, can_dump_time_us: _ }) {

                    let MachineSpec::Conveyor { item, filter, can_move_time_us, can_dump_time_us } = game_main.mach_data[mach.0].spec.clone() else { panic!(); };

                    let forward = dir_to_ivec2(&game_main.mach_data[mach.0].dir);

                    if item.count == 0 && can_move_time_us < game_main.time_us {

                        // take item from behind

                        let back = pos.as_ivec2() - forward;

                        //let mut convey: Option<(ItemTypeId, u32)> = None;

                        // tiles valid?
                        if    0 <= back.x  && back.x  < game_main.world_size.x as i32
                           && 0 <= back.y  && back.y  < game_main.world_size.y as i32 {

                            // item in back side?
                            let backopt = game_main.feral_by_tile.get(&(back.x as u8, back.y as u8));

                            if backopt.is_some() {

                                let back_feral = backopt.unwrap().clone();


                                // weh
                                if filter && item.itemtype != Default::default() {

                                    // extract specific item

                                    let MachineSpec::Conveyor { item, filter: _, can_move_time_us, can_dump_time_us } = &mut game_main.mach_data[mach.0].spec else { panic!(); };

                                    *can_move_time_us = game_main.time_us + CONVEY_TAKE_PERIOD;
                                    *can_dump_time_us = game_main.time_us + CONVEY_DUMP_PERIOD;

                                    let back_feral_d = &mut game_main.feral_data[back_feral.0];

                                    let taken = take_items(&mut back_feral_d.slots, item.itemtype, 1);

                                    if taken == 1 {
                                        item.count += 1;

                                        if back_feral_d.slots.iter().all(|x| x.is_none()) {
                                            game_main.feral_by_tile.remove(&(back.x as u8, back.y as u8));
                                            game_main.feral_ids.remove(back_feral);
                                        }
                                    }

                                } else {
                                    let MachineSpec::Conveyor { item, filter: _, can_move_time_us, can_dump_time_us } = &mut game_main.mach_data[mach.0].spec else { panic!(); };

                                    *can_move_time_us = game_main.time_us + CONVEY_TAKE_PERIOD;
                                    *can_dump_time_us = game_main.time_us + CONVEY_DUMP_PERIOD;

                                    let back_feral_d = &mut game_main.feral_data[back_feral.0];
                                    let slot_take = back_feral_d.slots[0].as_mut().unwrap();

                                    item.itemtype = slot_take.itemtype;
                                    item.count += 1;
                                    slot_take.count -= 1;

                                    if slot_take.count == 0 {
                                        back_feral_d.slots[0] = None;
                                        back_feral_d.slots.rotate_left(1);
                                        if back_feral_d.slots.iter().all(|x| x.is_none()) {
                                            game_main.feral_by_tile.remove(&(back.x as u8, back.y as u8));
                                            game_main.feral_ids.remove(back_feral);
                                        }
                                    }
                                }
                            }
                        }
                    } else if item.count != 0 && can_dump_time_us < game_main.time_us {

                        // dump item to front

                        let front = pos.as_ivec2() + forward;

                        if 0 <= front.x && front.x < game_main.world_size.x as i32
                           && 0 <= front.y && front.y < game_main.world_size.y as i32 {

                            if let Some(front_mach) = game_main.mach_by_tile.get(&(front.x as u8, front.y as u8)) {
                                match &mut game_main.mach_data[front_mach.0].spec {
                                    MachineSpec::Turret { ammo, can_fire_time_us: _ } => {
                                        if item.itemtype == ITEM_BULLET && *ammo < 69 {
                                            // Refill turret in front
                                            *ammo += 1;

                                            let MachineSpec::Conveyor { item, filter: _, can_move_time_us: _, can_dump_time_us} = &mut game_main.mach_data[mach.0].spec else { panic!(); };
                                            item.count -= 1;
                                            *can_dump_time_us = game_main.time_us + CONVEY_DUMP_PERIOD;
                                        }
                                    },
                                    MachineSpec::Conveyor { item: other_item, filter: _, can_move_time_us: _, can_dump_time_us: other_can_dump_time_us } => {
                                        if other_item.count == 0 {
                                            other_item.itemtype = item.itemtype;
                                            other_item.count += 1;

                                            *other_can_dump_time_us = game_main.time_us + CONVEY_DUMP_PERIOD;

                                            let MachineSpec::Conveyor { item, filter: _, can_move_time_us: _, can_dump_time_us} = &mut game_main.mach_data[mach.0].spec else { panic!(); };
                                            item.count -= 1;
                                            *can_dump_time_us = game_main.time_us + CONVEY_DUMP_PERIOD;
                                       }
                                    },
                                    _ => {}
                                }
                            } else if filter.not() && item.count == 1 && item.itemtype == ITEM_DEAD_DRONE && mq::gen_range(0, 20) == 1 {
                                // chance to disassemble drones

                                let place_success = place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, front.as_uvec2(), ItemSlot { itemtype: ITEM_SCRAP, count: 1 }).is_ok();

                                if mq::gen_range(0, 5) == 1 {
                                    // chance to make additional alignite
                                    place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, front.as_uvec2(), ItemSlot { itemtype: ITEM_ALIGNITE, count: 1 }).ok();
                                }

                                if place_success {

                                    let MachineSpec::Conveyor { item, filter: _, can_move_time_us: _, can_dump_time_us} = &mut game_main.mach_data[mach.0].spec else { panic!(); };
                                    item.itemtype = ITEM_BATTERY;

                                    *can_dump_time_us = game_main.time_us + CONVEY_DUMP_PERIOD;
                                }

                            } else if filter.not() && item.count == 1 && item.itemtype == ITEM_CLUMP && mq::gen_range(0, 200) == 1 {
                                // chance to misalign alignite clumps

                                let place_success = place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, front.as_uvec2(), ItemSlot { itemtype: ITEM_OBFUSCATOR, count: 1 }).is_ok();

                                if place_success {
                                    // all good, all 1 items have been dispensed and nothing exploded
                                    let MachineSpec::Conveyor { item, filter: _, can_move_time_us: _, can_dump_time_us} = &mut game_main.mach_data[mach.0].spec else { panic!(); };
                                    item.count -= 1;

                                    *can_dump_time_us = game_main.time_us + CONVEY_DUMP_PERIOD;
                                }

                            } else {
                                let place_success = place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, front.as_uvec2(), ItemSlot { itemtype: item.itemtype, count: 1 }).is_ok();

                                if place_success {
                                    // all good, all 1 items have been dispensed and nothing exploded
                                    let MachineSpec::Conveyor { item, filter: _, can_move_time_us: _, can_dump_time_us} = &mut game_main.mach_data[mach.0].spec else { panic!(); };
                                    item.count -= 1;

                                    *can_dump_time_us = game_main.time_us + CONVEY_DUMP_PERIOD;
                                }
                            }
                        }
                    }
                }
            }
        }


        // Player Shoot
        if game_main.player_gun_ammo != 0 && matches!(game_main.tool, ToolMode::GunPod) && mq::is_mouse_button_down(mq::MouseButton::Left) {

            if game_main.player_gun_cooldown <= 0.0 {

                let mouse_dir = (game_draw.mouse_pos - game_main.player_pos).normalize();
                let dir = Mat2::from_cols(mouse_dir, rot_cw_90(mouse_dir)).mul_vec2(vec2(100.0, (mq::gen_range(-1.0, 1.0) as f32).powf(3.0)).normalize());

                mq::stop_sound(&shoot0_sound);
                mq::play_sound(&shoot0_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });

                let bullet = game_main.bullet_ids.create().unwrap();
                game_main.bullet_pos[bullet.0]  = game_main.player_pos + dir * 0.5*TILE_SIZE;
                game_main.bullet_data[bullet.0] = Bullet{dir, speed: 1200.0, travel: 0.0, travel_max: 800.0};

                game_main.player_gun_cooldown += 0.4 - 0.25 * f32::min(1.0, (game_main.player_gun_consecutive as f32) / 12.0).powf(0.5);
                game_main.player_gun_consecutive += 1;
                game_main.player_gun_ammo -= 1;
            }
        } else {
            if game_main.player_gun_cooldown <= 0.0 {
                game_main.player_gun_cooldown = 0.0;
                game_main.player_gun_consecutive = 0;
            }
        }

        game_main.player_gun_cooldown -= delta;

        //frame_count += 1;

        game_main.time_us += (delta * 1000000.0) as u64;

        mq::next_frame().await
    }
}

