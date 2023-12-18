

use obfuscation::lgrn;
use obfuscation::game;
use obfuscation::draw;

use glam::{vec2, Mat4, Vec2};

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

    let mut step_sounds: Vec<mq::Sound> = Vec::with_capacity(4);
    step_sounds.push(mq::load_sound_from_bytes(&data).await.unwrap());
    repitch(&mut data, 44100, 0.8);
    step_sounds.push(mq::load_sound_from_bytes(&data).await.unwrap());
    repitch(&mut data, 44100, 1.1);
    step_sounds.push(mq::load_sound_from_bytes(&data).await.unwrap());
    repitch(&mut data, 44100, 0.9);
    step_sounds.push(mq::load_sound_from_bytes(&data).await.unwrap());

    //let step: mq::Sound;
    //step = mq::load_sound_from_bytes(&data).await.unwrap();


    let mut game_main = game::GameMain
    {
        player_pos: vec2(2.0, 2.0)
    };


    let mut game_draw: draw::GameDraw = draw::make_game_draw().await;



    lgrn::bitvec_set(&mut a, 2);

    assert!(a[0] & 4 != 0);

    let mut b: u32 = 2;



    loop {

        if b % 10 == 1
        {
            mq::play_sound_once(&step_sounds[mq::gen_range(0, step_sounds.len())]);
        }


        b += 1;

        draw::draw_game(&game_main, &mut game_draw);

        mq::next_frame().await
    }
}
