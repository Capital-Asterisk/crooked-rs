mod lgrn;

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



    let texture: mq::Texture2D = mq::load_texture("tf/custom/rook1.png").await.unwrap();

    lgrn::bitvec_set(&mut a, 2);

    assert!(a[0] & 4 != 0);

    let mut b: u32 = 2;



    loop {

        if b % 10 == 1
        {
            mq::play_sound_once(&step_sounds[mq::gen_range(0, step_sounds.len())]);
        }

        mq::clear_background(mq::Color::from_rgba(1, 46, 87, 255));




        //mq::draw_line(40.0, 40.0, 100.0, 200.0, 15.0, mq::BLUE);
        //mq::draw_rectangle(mq::screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, mq::GREEN);
        //mq::draw_circle(mq::screen_width() - 30.0, mq::screen_height() - 30.0, 15.0, mq::YELLOW);

        //mq::draw_text("IT WORKS!", 20.0, b as f32, 30.0, mq::DARKGRAY);
        mq::draw_texture(&texture, b as f32, 100.0 - (((mq::get_time() as f32)*2.0*3.14159*2.0).sin().abs()*40.0)+b as f32, mq::WHITE);

        b += 1;

        mq::next_frame().await
    }
}
