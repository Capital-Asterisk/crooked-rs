

use std::ops::Not;

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

    game_main.drone_ids.resize(512);
    game_main.drone_pos.resize(512, vec2(0.0, 0.0));
    game_main.drone_data.resize(512, game::DroneData{rail_idx: 0, rail_pos: 0.0, speed: 0.0});



    let mut game_draw: draw::GameDraw = draw::make_game_draw().await;

    let mut remove_drones: Vec<game::DroneId> = Vec::with_capacity(8);

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
                    Bend::Right   => vec2(-dir.y, dir.x),
                    Bend::Left    => vec2(dir.y, -dir.x),
                };
            }

            *p = (r.pos.as_vec2() + vec2(0.5, 0.5-0.125) + dir * (d.rail_pos - 0.5)) * game::TILE_SIZE;

            if matches!(r.bend, Bend::Forward).not() && d.rail_pos < 0.5 && d.rail_pos+d.speed*delta > 0.5 {
                d.rail_pos = 0.5;
                d.speed = 0.0;
            } else {
                d.rail_pos += d.speed * delta;
                d.speed += delta*2.0;
            }

            if d.rail_pos > 1.0 {
                d.rail_pos -= 1.0;
                d.rail_idx += 1;

                if d.rail_idx == game_main.rail.len() {
                    remove_drones.push(drone);
                }
            }
        }

        // Remove drones
        for drone in &remove_drones {
            game_main.drone_ids.remove(*drone);
        }
        remove_drones.clear();

        if frame_count % 30 == 0 {
            let a = game_main.drone_ids.create().unwrap();
            game_main.drone_pos[a.0]  = vec2(0.0, 4.5) * game::TILE_SIZE;
            game_main.drone_data[a.0] = game::DroneData{rail_idx: 0, rail_pos: 0.0, speed: 3.0};
        }


        frame_count += 1;

        mq::next_frame().await
    }
}
