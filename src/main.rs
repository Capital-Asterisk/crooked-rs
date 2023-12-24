use std::cmp::Ordering;
use std::ops::Not;

use glam::Mat2;
use obfuscation::game::GameMain;
//use obfuscation::lgrn;
use obfuscation::game;
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

    //let step: mq::Sound;
    //step = mq::load_sound_from_bytes(&data).await.unwrap();

    let mut controls = game::Controls
    {
        walk: vec2(0.0, 0.0)
    };

    let mut game_main: GameMain = Default::default();

    game_main.world_size = uvec2(80, 80);

    use game::Dir;
    use game::Bend;

    for x in 0..10 {
        game_main.rail.push(game::Rail { pos: uvec2(x, 4), dir: game::Dir::Right, bend: Bend::Forward});
    }
    game_main.rail.push(game::Rail { pos: uvec2(10, 4), dir: Dir::Right, bend: Bend::Right});
    game_main.rail.push(game::Rail { pos: uvec2(10, 5), dir: Dir::Down,  bend: Bend::Forward});
    game_main.rail.push(game::Rail { pos: uvec2(10, 6), dir: Dir::Down,  bend: Bend::Forward});
    game_main.rail.push(game::Rail { pos: uvec2(10, 7), dir: Dir::Down,  bend: Bend::Forward});
    game_main.rail.push(game::Rail { pos: uvec2(10, 8), dir: Dir::Down,  bend: Bend::Right});
    game_main.rail.push(game::Rail { pos: uvec2(9,  8), dir: Dir::Left,  bend: Bend::Forward});
    game_main.rail.push(game::Rail { pos: uvec2(8,  8), dir: Dir::Left,  bend: Bend::Right});
    game_main.rail.push(game::Rail { pos: uvec2(8,  7), dir: Dir::Up,    bend: Bend::Left});
    game_main.rail.push(game::Rail { pos: uvec2(7,  7), dir: Dir::Left,  bend: Bend::Left});
    game_main.rail.push(game::Rail { pos: uvec2(7,  8), dir: Dir::Down,  bend: Bend::Forward});

    for y in 9..48 {
        game_main.rail.push(game::Rail { pos: uvec2(7,  y), dir: Dir::Down, bend: Bend::Forward});
    }

    game_main.drone_ids.resize(512);
    game_main.drone_pos.resize(512, vec2(0.0, 0.0));
    game_main.drone_data.resize(512, game::DroneData{rail_idx: 0, rail_pos: 0.0, speed: 0.0});
    game_main.drone_by_x.reserve(512);


    game_main.bullet_ids.resize(128);
    game_main.bullet_pos.resize(512, vec2(0.0, 0.0));
    game_main.bullet_data.resize(512, game::BulletData{dir: vec2(0.0, 0.0), speed: 0.0, travel: 0.0, travel_max: 0.0});



    let mut game_draw: draw::GameDraw = draw::make_game_draw().await;





    let mut frame_count: u64 = 0;

    loop {

        let delta: f32 = mq::get_frame_time();

        controls.walk.x = (mq::is_key_down(mq::KeyCode::D) as i32 - mq::is_key_down(mq::KeyCode::A) as i32) as f32;
        controls.walk.y = (mq::is_key_down(mq::KeyCode::S) as i32 - mq::is_key_down(mq::KeyCode::W) as i32) as f32;
        let is_walking = controls.walk.length_squared() > 0.01;

        draw::draw_game(&game_main, &mut game_draw);

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
        game_main.player_pos += controls.walk * delta * game::TILE_SIZE.x * 5.0;



        use game::Dir;
        use game::Bend;

        // Remove drones
        for drone in &game_main.remove_drones {
            game_main.drone_ids.remove(*drone);
        }
        game_main.remove_drones.clear();

        // Move drones
        for drone in game_main.drone_ids.iter_ids() {
            let d: &mut game::DroneData = &mut game_main.drone_data[drone.0];
            let p: &mut Vec2            = &mut game_main.drone_pos[drone.0];
            let r: &game::Rail          = &game_main.rail[d.rail_idx];

            let mut dir = match r.dir {
                Dir::Right => vec2( 1.0,  0.0),
                Dir::Down  => vec2( 0.0,  1.0),
                Dir::Left  => vec2(-1.0,  0.0),
                Dir::Up    => vec2( 0.0, -1.0),
            };

            if d.rail_pos > 0.5 {
                dir = match r.bend {
                    Bend::Forward => dir,
                    Bend::Right   => game::rot_cw_90(dir),
                    Bend::Left    => game::rot_ccw_90(dir)
                };
            }

            *p = (r.pos.as_vec2() + vec2(0.5, 0.5-0.125) + dir * (d.rail_pos - 0.5)) * game::TILE_SIZE;


            if matches!(r.bend, Bend::Forward).not() && d.rail_pos < 0.5 && d.rail_pos+d.speed*delta > 0.5 {
                d.rail_pos = 0.5;
                d.speed *= 0.5;

                let vol = (1.0 - (game_draw.cam_center - *p).length()/8.0/game::TILE_SIZE.x).min(1.0) * 0.2;
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

        if frame_count % 20 == 0 {
            let id = game_main.drone_ids.create().unwrap();
            let pos = &mut game_main.drone_pos[id.0];
            *pos = vec2(0.0, 4.5) * game::TILE_SIZE;
            game_main.drone_data[id.0] = game::DroneData{rail_idx: 0, rail_pos: 0.0, speed: 3.0};
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
            let d: &mut game::BulletData = &mut game_main.bullet_data[id.0];
            let p: &mut Vec2             = &mut game_main.bullet_pos[id.0];

            let trav = d.speed * delta;
            d.travel += trav;

            if d.travel > d.travel_max {
                game_main.remove_bullets.push(id);
            }

            let max_x: f32 = p.x - 0.5*game::TILE_SIZE.x;

            // lower_bound: https://stackoverflow.com/questions/75790347/
            let mut idx = game_main.drone_by_x.binary_search_by(|x| match x.1.total_cmp(&max_x) {
                Ordering::Equal => Ordering::Greater,
                ord => ord,
            }).unwrap_err();

            //println!("wot: {} {}", idx, game_main.drone_by_x.len());


            while idx < game_main.drone_by_x.len() {
                let drone_id = game_main.drone_by_x[idx].0;
                let drone_pos = game_main.drone_pos[drone_id.0];
                let drone_tl = drone_pos - game::TILE_SIZE * 0.5;
                let drone_br = drone_pos + game::TILE_SIZE * 0.5;

                if    drone_tl.x < p.x && p.x < drone_br.x
                   && drone_tl.y < p.y && p.y < drone_br.y {

                    let drone_tr = vec2(drone_br.x, drone_tl.y);
                    let drone_bl = vec2(drone_tl.x, drone_br.y);

                    let norm = (|| {

                        if d.dir.x < 0.0 {
                            if game::line_segment_vs_line_intersect((drone_tr, drone_br), *p, d.dir) {
                                return Some(vec2(1.0, 0.0));
                            }
                        } else if 0.0 < d.dir.x {
                            if game::line_segment_vs_line_intersect((drone_tl, drone_bl), *p, d.dir) {
                                return Some(vec2(-1.0, 0.0));
                            }
                        }

                        if d.dir.y < 0.0 {
                            if game::line_segment_vs_line_intersect((drone_bl, drone_br), *p, d.dir) {
                                return Some(vec2(0.0, 1.0));
                            }
                        } else if 0.0 < d.dir.y {
                            if game::line_segment_vs_line_intersect((drone_tl, drone_tr), *p, d.dir) {
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
                        } else {
                            d.dir = d.dir + 2.0*norm*dot;
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
        if mq::is_mouse_button_down(mq::MouseButton::Left) && frame_count % 8 == 0 {
            let id = game_main.bullet_ids.create().unwrap();

            let mouse_dir = (game_draw.mouse_pos - game_main.player_pos).normalize();
            let dir = Mat2::from_cols(mouse_dir, game::rot_cw_90(mouse_dir)).mul_vec2(vec2(10.0, (mq::gen_range(-1.0, 1.0) as f32).powf(3.0)).normalize());

            mq::stop_sound(&shoot0_sound);
            mq::play_sound(&shoot0_sound, mq::PlaySoundParams { looped: false, volume: 0.5 });

            game_main.bullet_pos[id.0]  = game_main.player_pos + dir * 0.5*game::TILE_SIZE;
            game_main.bullet_data[id.0] = game::BulletData{dir, speed: 1000.0, travel: 0.0, travel_max: 800.0};
        }



        frame_count += 1;

        mq::next_frame().await
    }
}
