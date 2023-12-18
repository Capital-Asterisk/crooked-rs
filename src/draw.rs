use crate::game;

pub struct GameDraw
{
    pub rook1: mq::Texture2D
}

pub mod mq
{
    pub use macroquad::prelude::*;
    pub use macroquad::audio::*;
    pub use macroquad::rand::*;
}

pub async fn make_game_draw() -> GameDraw
{

    GameDraw{
        rook1: mq::load_texture("tf/custom/rook1.png").await.unwrap()
    }
}

pub fn draw_game(main: &game::GameMain, draw: &mut GameDraw)
{
    mq::clear_background(mq::Color::from_rgba(1, 46, 87, 255));

    mq::draw_texture(&draw.rook1, main.player_pos.x, 100.0 - (((mq::get_time() as f32)*2.0*3.14159*2.0).sin().abs()*40.0)+main.player_pos.y, mq::WHITE);
}
