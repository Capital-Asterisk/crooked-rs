

use obfuscation::game::GameMain;
use obfuscation::lgrn;
use obfuscation::game;
use obfuscation::draw;

use glam::{vec2, Vec2, uvec2};

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



    let mut a: lgrn::BitVec = vec![0, 0];

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

    for x in 0..80 {
        game_main.rail.push(game::Rail { pos: uvec2(x, 4), dir: game::Dir::Right, bend: false});
    }



    let mut game_draw: draw::GameDraw = draw::make_game_draw().await;



    lgrn::bitvec_set(&mut a, 2);

    assert!(a[0] & 4 != 0);

    loop {


        controls.walk.x = (mq::is_key_down(mq::KeyCode::D) as i32 - mq::is_key_down(mq::KeyCode::A) as i32) as f32;
        controls.walk.y = (mq::is_key_down(mq::KeyCode::S) as i32 - mq::is_key_down(mq::KeyCode::W) as i32) as f32;
        let is_walking = controls.walk.length_squared() > 0.01;

        draw::draw_game(&game_main, &mut game_draw);

        game_draw.player_hop_time += mq::get_frame_time();

        if is_walking && game_draw.player_hop_time > game_draw.player_hop_rate {
            game_main.hop_count += 1;
            game_draw.player_hop_time = 0.0;
            mq::play_sound_once(&step_sounds[mq::gen_range(0, step_sounds.len())]);
        }



        if controls.walk.x.abs() > 0.01 {
            game_main.player_facing = controls.walk.x.signum() as i8;
        }

        game_main.player_pos += controls.walk * 5.0;

        mq::next_frame().await
    }
}
