use std::cmp::Ordering;
use std::ops::Not;

use glam::Mat2;
use obfuscation::draw::TileThing;
use obfuscation::game::GameMain;
use obfuscation::game::*;
use obfuscation::draw;

use glam::{Vec2, vec2, uvec2};

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

#[macroquad::main("BasicShapes")]
async fn main() {


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
    let pickup_sound = mq::load_sound("tf/custom/pickup.wav").await.unwrap();
    let place_sound = mq::load_sound("tf/custom/place.wav").await.unwrap();
    let deflect_sound = mq::load_sound("tf/custom/deflect.wav").await.unwrap();
    let explode_sound = mq::load_sound("tf/custom/explode.wav").await.unwrap();
    let craft_sound = mq::load_sound("tf/custom/craft.wav").await.unwrap();

    //let step: mq::Sound;
    //step = mq::load_sound_from_bytes(&data).await.unwrap();

    let mut controls = Controls
    {
        walk: vec2(0.0, 0.0)
    };

    let mut game_main: GameMain = Default::default();

    game_main.world_size = uvec2(80, 80);

    for x in 0..10 {
        game_main.rail.push(Rail { pos: uvec2(x, 4), dir: Dir::Right, bend: Bend::Forward});
    }
    game_main.rail.push(Rail { pos: uvec2(10, 4), dir: Dir::Right, bend: Bend::Right});
    game_main.rail.push(Rail { pos: uvec2(10, 5), dir: Dir::Down,  bend: Bend::Forward});
    game_main.rail.push(Rail { pos: uvec2(10, 6), dir: Dir::Down,  bend: Bend::Forward});
    game_main.rail.push(Rail { pos: uvec2(10, 7), dir: Dir::Down,  bend: Bend::Forward});
    game_main.rail.push(Rail { pos: uvec2(10, 8), dir: Dir::Down,  bend: Bend::Right});
    game_main.rail.push(Rail { pos: uvec2(9,  8), dir: Dir::Left,  bend: Bend::Forward});
    game_main.rail.push(Rail { pos: uvec2(8,  8), dir: Dir::Left,  bend: Bend::Right});
    game_main.rail.push(Rail { pos: uvec2(8,  7), dir: Dir::Up,    bend: Bend::Left});
    game_main.rail.push(Rail { pos: uvec2(7,  7), dir: Dir::Left,  bend: Bend::Left});
    game_main.rail.push(Rail { pos: uvec2(7,  8), dir: Dir::Down,  bend: Bend::Forward});

    for y in 9..48 {
        game_main.rail.push(Rail { pos: uvec2(7,  y), dir: Dir::Down, bend: Bend::Forward});
    }

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


    game_main.feral_ids.resize(512);
    let j = game_main.feral_ids.create().unwrap();

    game_main.feral_data.resize(512, Default::default());
    game_main.feral_data[j.0].slots[0] = Some(ItemSlot{itemtype: ItemTypeId(0), count: 1});
    game_main.feral_by_tile.insert((0, 0), j);

    place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(3, 4), ItemSlot { itemtype: ItemTypeId(2), count: 44 }).ok();
    place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(3, 4), ItemSlot { itemtype: ItemTypeId(2), count: 44 }).ok();
    place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(3, 4), ItemSlot { itemtype: ItemTypeId(2), count: 44 }).ok();
    place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, uvec2(3, 4), ItemSlot { itemtype: ItemTypeId(2), count: 44 }).ok();

    let mut game_draw: draw::GameDraw = draw::make_game_draw().await;

    let mut frame_count: u64 = 0;

    loop {

        let delta: f32 = mq::get_frame_time();

        controls.walk.x = (mq::is_key_down(mq::KeyCode::D) as i32 - mq::is_key_down(mq::KeyCode::A) as i32) as f32;
        controls.walk.y = (mq::is_key_down(mq::KeyCode::S) as i32 - mq::is_key_down(mq::KeyCode::W) as i32) as f32;
        let is_walking = controls.walk.length_squared() > 0.01;


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

        // Pick up and place items
        if let ToolMode::Construct(drag) = &mut game_main.tool {
            if mq::is_mouse_button_pressed(mq::MouseButton::Left)
               || mq::is_key_pressed(mq::KeyCode::E) {
                if let Drag::None = drag {
                    if let TileThing::Feral(feral) = game_draw.under_cursor {
                        let d = &mut game_main.feral_data[feral.0];

                        let slot = &mut d.slots[0]; //&mut d.slots.iter_mut().find(|s| s.is_some()).unwrap();

                        let slot_extracted = slot.take().unwrap();
                        d.slots.rotate_left(1);

                        feral_remove_if_empty(&mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, feral);

                        *drag = Drag::Item(slot_extracted);

                        mq::play_sound(&pickup_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });
                    }
                } else if matches!(drag, Drag::Item(_)) {

                    let valid_placement = true;

                    let Drag::Item(slot) = std::mem::take(drag) else { panic!() };

                    if valid_placement {
                        match place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, game_draw.mouse_select, slot) {
                            Ok(feral) => {
                                mq::play_sound(&place_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });
                            },
                            Err((feral, slot)) => {
                                *drag = Drag::Item(slot);
                            }
                        }
                    }
                }
            }

            if let Drag::None = drag {

                if mq::is_key_pressed(mq::KeyCode::Key1) {
                    if let TileThing::Feral(feral) = game_draw.under_cursor {
                        if let Some(gwah) = craft_item_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 0) {
                            game_main.feral_data[feral.0].slots = gwah;
                            mq::play_sound(&craft_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });
                        }
                    }
                } else if mq::is_key_pressed(mq::KeyCode::Key5) {
                    if let TileThing::Feral(feral) = game_draw.under_cursor {
                        if let Some(gwah) = craft_machine_recipe(&game_main.itemtype_data, &game_main.feral_data[feral.0].slots, 5) {
                            game_main.feral_data[feral.0].slots = gwah;
                            mq::play_sound(&craft_sound, mq::PlaySoundParams { looped: false, volume: 1.0 });

                            feral_remove_if_empty(&mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, feral);
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
            let r: &Rail          = &game_main.rail[d.rail_idx];

            let mut dir = match r.dir {
                Dir::Right => vec2( 1.0,  0.0),
                Dir::Down  => vec2( 0.0,  1.0),
                Dir::Left  => vec2(-1.0,  0.0),
                Dir::Up    => vec2( 0.0, -1.0),
            };

            if d.rail_pos > 0.5 {
                dir = match r.bend {
                    Bend::Forward => dir,
                    Bend::Right   => rot_cw_90(dir),
                    Bend::Left    => rot_ccw_90(dir)
                };
            }

            *p = (r.pos.as_vec2() + vec2(0.5, 0.5-0.125) + dir * (d.rail_pos - 0.5)) * TILE_SIZE;


            if matches!(r.bend, Bend::Forward).not() && d.rail_pos < 0.5 && d.rail_pos+d.speed*delta > 0.5 {
                d.rail_pos = 0.5;
                d.speed *= 0.5;

                let vol = (1.0 - (game_draw.cam_center - *p).length()/8.0/TILE_SIZE.x).min(1.0) * 0.2;
                if vol > 0.0 {
                    mq::play_sound(&changedir_sound, mq::PlaySoundParams { looped: false, volume: vol });
                }

            } else {
                d.rail_pos += d.speed * delta;
                d.speed = f32::min(d.speed + delta*2.0, 5.0);
            }

            if d.rail_pos > 1.0 {
                d.rail_pos -= 1.0;
                d.rail_idx += 1;

                if d.rail_idx == game_main.rail.len() {
                    game_main.remove_drones.push(drone);
                }
            }
        }

        if frame_count % 50 == 0 {
            let id = game_main.drone_ids.create().unwrap();
            let pos = &mut game_main.drone_pos[id.0];
            *pos = vec2(0.0, 4.5) * TILE_SIZE;
            game_main.drone_data[id.0] = Drone{rail_idx: 0, rail_pos: 0.0, speed: 3.0};
            game_main.drone_by_x.push((id, pos.x));
        }

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

            let max_x: f32 = p.x - 0.5*TILE_SIZE.x;

            // lower_bound: https://stackoverflow.com/questions/75790347/
            let mut idx = game_main.drone_by_x.binary_search_by(|x| match x.1.total_cmp(&max_x) {
                Ordering::Equal => Ordering::Greater,
                ord => ord,
            }).unwrap_err();

            //println!("wot: {} {}", idx, game_main.drone_by_x.len());


            while idx < game_main.drone_by_x.len() {
                let drone_id = game_main.drone_by_x[idx].0;
                let drone_pos = game_main.drone_pos[drone_id.0];
                let drone_tl = drone_pos - TILE_SIZE * 0.5;
                let drone_br = drone_pos + TILE_SIZE * 0.5;

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


                        if dot > f32::cos(26.0_f32.to_radians()) {
                            game_main.remove_drones.push(drone_id);
                            game_main.remove_bullets.push(id);
                            place_item(&game_main.itemtype_data, &mut game_main.feral_ids, &mut game_main.feral_data, &mut game_main.feral_by_tile, (drone_pos / TILE_SIZE).floor().as_uvec2(),
                               ItemSlot { itemtype: ItemTypeId(0), count: 1 }).ok();
                            mq::play_sound(&explode_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });
                        } else {
                            d.dir = d.dir + 2.0*norm*dot;
                            mq::play_sound(&deflect_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });
                        }
                    }


                    println!( "AAAAAAA: {}", norm.unwrap_or(vec2(69.0, 69.0)));

                    break;
                }

                if p.x < drone_br.x {
                    idx += 1;
                } else {
                    break;
                }
            }

            *p += d.dir * trav;

        }

        // Shoot
        if matches!(game_main.tool, ToolMode::GunPod) && mq::is_mouse_button_down(mq::MouseButton::Left) && frame_count % 8 == 0 {
            let id = game_main.bullet_ids.create().unwrap();

            let mouse_dir = (game_draw.mouse_pos - game_main.player_pos).normalize();
            let dir = Mat2::from_cols(mouse_dir, rot_cw_90(mouse_dir)).mul_vec2(vec2(100.0, (mq::gen_range(-1.0, 1.0) as f32).powf(3.0)).normalize());

            mq::stop_sound(&shoot0_sound);
            mq::play_sound(&shoot0_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });

            game_main.bullet_pos[id.0]  = game_main.player_pos + dir * 0.5*TILE_SIZE;
            game_main.bullet_data[id.0] = Bullet{dir, speed: 1200.0, travel: 0.0, travel_max: 800.0};
        }



        frame_count += 1;

        mq::next_frame().await
    }
}

