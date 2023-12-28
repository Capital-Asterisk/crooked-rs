use crate::game::*;


use glam::{vec2, Vec2, uvec2, UVec2, Mat2, mat2};

use std::ops::Not;



pub struct GameDraw {
    pub sprites: mq::Texture2D,

    pub cam_center: Vec2,
    pub mouse_pos: Vec2,
    pub mouse_select: UVec2,
    pub under_cursor: TileThing,

    pub stupidraw: Vec<(f32, Vec2, Mat2, (Vec2,Vec2))>,

    pub player_hop_time: f32,
    pub player_hop_rate: f32,
    pub player_hop_height: f32,

    pub clock_1s: f32
}

pub enum TileThing {
    None, Feral(FeralItemId), Machine(MachineId)
}

pub mod mq {
    pub use macroquad::prelude::*;
    pub use macroquad::audio::*;
    pub use macroquad::rand::*;
    pub use macroquad::texture::*;
}

pub async fn make_game_draw() -> GameDraw {
    GameDraw{
        sprites:            mq::load_texture("tf/custom/sprites.png").await.unwrap(),
        cam_center:         vec2(0.0, 0.0),
        mouse_pos:          vec2(0.0, 0.0),
        mouse_select:       uvec2(0, 0),
        under_cursor:       TileThing::None,
        stupidraw:          Default::default(),
        player_hop_time:    0.0,
        player_hop_rate:    0.25,
        player_hop_height:  35.0,
        clock_1s:           0.0
    }
}

pub fn sprite(x: i32, y: i32) -> (Vec2, Vec2) {
    let square:  f32 = 0.125;     // 512.0/4096.0;
    let padding: f32 = 0.0078125; // 32.0/4096.0;

    let c = vec2(x as f32, y as f32);

    let top_left = c * (square + padding);

    (top_left, top_left + vec2(square, square))
}

pub fn flip_x(a: (Vec2, Vec2)) -> (Vec2, Vec2) {
    (vec2(a.1.x, a.0.y), vec2(a.0.x, a.1.y))
}

pub fn flip_y(a: (Vec2, Vec2)) -> (Vec2, Vec2) {
    (vec2(a.0.x, a.1.y), vec2(a.1.x, a.0.y))
}

pub fn draw_game(main: &GameMain, draw: &mut GameDraw) {
    let view_size = TILE_SIZE.x * 10.0;

    let world_size_f32 = main.world_size.as_vec2() * TILE_SIZE;

    // Do camera stuff
    let screen_size = vec2(mq::screen_width(), mq::screen_height());
    draw.cam_center = main.player_pos.clamp(0.5*vec2(view_size, view_size), world_size_f32 - 0.5*vec2(view_size, view_size));
    let screen_wide = screen_size.x > screen_size.y; // if false, screen is tall or square
    let view_square = if screen_wide { screen_size.y } else { screen_size.x };
    let view_scale = view_square / view_size;
    let view_offset = if screen_wide {
        vec2(0.5*screen_size.x - 0.5*screen_size.y, 0.0)
    } else {
        vec2(0.0, 0.5*screen_size.y - 0.5*screen_size.x)
    };
    let view_offset = view_offset - (draw.cam_center - 0.5*vec2(view_size, view_size)) * view_scale;

    // Mouse
    let (mouse_x, mouse_y) = mq::mouse_position();
    draw.mouse_pos = (vec2(mouse_x, mouse_y) - view_offset) / view_scale;
    draw.mouse_select = (draw.mouse_pos / TILE_SIZE).floor().as_uvec2();

    // what's under the cursor?
    draw.under_cursor = (|| {
        if matches!(main.tool, ToolMode::Construct(_)) {
            if let Some(feral) = main.feral_by_tile.get(&(draw.mouse_select.x as u8, draw.mouse_select.y as u8)) {
                return TileThing::Feral(*feral);
            }
        }
        return TileThing::None;
    })();

    // DRAW!

    mq::clear_background(mq::Color::from_rgba(1, 46, 87, 255));

    // Draw rail
    for rail in &main.rail {

        let pos = (rail.pos.as_vec2() + 0.5) * TILE_SIZE * view_scale + view_offset;

        let mat = match rail.dir {
            Dir::Right => Mat2::from_cols(vec2( 1.0,  0.0), vec2( 0.0,  1.0)),
            Dir::Down  => Mat2::from_cols(vec2( 0.0,  1.0), vec2(-1.0,  0.0)),
            Dir::Left  => Mat2::from_cols(vec2(-1.0,  0.0), vec2( 0.0, -1.0)),
            Dir::Up    => Mat2::from_cols(vec2( 0.0, -1.0), vec2( 1.0,  0.0))
        };

        let mat = mat * Mat2::from_diagonal(TILE_SIZE * view_scale);

        let coord = match rail.bend {
            Bend::Forward => sprite(0, 2),
            Bend::Right   => sprite(1, 2),
            Bend::Left    => flip_y(sprite(1, 2))
        };

        draw_texture_gwah_checked(&draw.sprites, pos, mat, coord, mq::WHITE);
    }

    // Draw drones
    for id in main.drone_ids.iter_ids() {

        let pos = main.drone_pos[id.0] * view_scale + view_offset;

        let mat = Mat2::from_diagonal(TILE_SIZE * view_scale);

        if on_screen_mat(pos, mat) {
            draw.stupidraw.push((pos.y, pos, mat, sprite(1, 0)));
        }
    }

    // Draw player
    {
        let hnorm: f32 = (draw.player_hop_time/draw.player_hop_rate).min(1.0);
        let hop: f32   = (1.0 - 4.0*(hnorm-0.5).powi(2)).max(0.0) * draw.player_hop_height;

        let hop_rot: f32 = if hnorm == 1.0 {
            0.0
        } else {
            hnorm * 0.4 * (if main.hop_count % 2 == 0 {1.0} else {-1.0} )
        };

        let coord = if main.player_facing == 1 { sprite(0, 0) } else { flip_x(sprite(0, 0)) };
        let pos = main.player_pos * view_scale + view_offset;
        let mat = Mat2::from_scale_angle(TILE_SIZE * view_scale, hop_rot);

        draw.stupidraw.push((pos.y, pos + vec2(0.0, -hop) * view_scale, mat, coord));
    }

    // Draw bullets
    for id in main.bullet_ids.iter_ids() {

        let dir = main.bullet_data[id.0].dir;
        let pos = -dir * TILE_SIZE*0.4 + main.bullet_pos[id.0] * view_scale + view_offset;


        let mat = Mat2::from_diagonal(TILE_SIZE * view_scale) * Mat2::from_cols(dir, rot_cw_90(dir));

        if on_screen_mat(pos, mat)
        {
            draw.stupidraw.push((pos.y, pos, mat, sprite(0, 1)));
        }
    }

    // Draw feral items
    for id in main.feral_ids.iter_ids() {

        let d = &main.feral_data[id.0];

        let dpos = (d.pos.as_vec2() + 0.5) * TILE_SIZE * view_scale + view_offset;

        let mat = Mat2::from_diagonal(TILE_SIZE * view_scale);

        if on_screen(dpos, TILE_SIZE * 2.0).not() {
            continue;
        }

        let count = d.slots.iter().filter(|a| a.is_some()).count();

        let mut draw_it_uwu = |idx: usize, offset: Vec2| {
            let Some((asdf, _)) = &d.slots[idx] else { panic!() };
            let dit = &main.itemtype_data[asdf.itemtype.0];
            draw.stupidraw.push((dpos.y, dpos + offset*TILE_SIZE, mat, dit.sprite));
        };

        match count {
            1 => {
                draw_it_uwu(0, vec2(0.0, 0.0));
            },
            2 => {
                draw_it_uwu(0, vec2(-0.1, -0.1));
                draw_it_uwu(1, vec2(0.1, 0.0));
            },
            3 => {
                draw_it_uwu(0, vec2(0.1, -0.1));
                draw_it_uwu(1, vec2(-0.1, 0.0));
                draw_it_uwu(2, vec2(0.0, 0.1));
            },
            4 => {
                draw_it_uwu(0, vec2(0.1, -0.1));
                draw_it_uwu(1, vec2(-0.1, -0.05));
                draw_it_uwu(2, vec2(0.1, 0.05));
                draw_it_uwu(3, vec2(-0.1, 0.1));
            },

            _ => {
                panic!();
            }
        };
    }

    // Draw sprites
    draw.stupidraw.sort_by(|lhs, rhs| lhs.0.partial_cmp(&rhs.0).unwrap() );
    for args in &draw.stupidraw {
        draw_texture_gwah(&draw.sprites, args.1, args.2, args.3, mq::WHITE);
    }
    draw.stupidraw.clear();

    //use std::f32::consts::PI;
    //+ 4.0*f32::sin(4.0*2.0*PI*draw.clock_1s)

    // Draw cursor


    let select_pos = draw.mouse_select.as_vec2() * TILE_SIZE * view_scale + view_offset;
    let select_size = TILE_SIZE * view_scale;
    match &main.tool {
        ToolMode::Construct(drag) => {
            match drag {
                Drag::None => {
                    let (color, thicc) = match draw.under_cursor {
                        TileThing::None => (mq::WHITE, 2.0),
                        TileThing::Feral(_) => (mq::GREEN, 8.0),
                        TileThing::Machine(_) => (mq::GREEN, 8.0)
                    };
                    mq::draw_rectangle_lines(select_pos.x, select_pos.y, select_size.x, select_size.y, thicc, color);
                },
                Drag::Item(slot) => {
                    let dit = &main.itemtype_data[slot.itemtype.0];
                    let mat = Mat2::from_diagonal(TILE_SIZE * view_scale);
                    draw_texture_gwah(&draw.sprites, vec2(select_pos.x, select_pos.y) + 0.5*TILE_SIZE*view_scale, mat, sprite(4, 1), mq::WHITE);
                    draw_texture_gwah(&draw.sprites, vec2(mouse_x, mouse_y) - vec2(0.0, 0.5*TILE_SIZE.y), mat, dit.sprite, mq::Color::new(1.0, 1.0, 1.0, 0.75));



                },
                _ => {}
            }
        },
        ToolMode::GunPod => {
            draw_texture_gwah(&draw.sprites, vec2(mouse_x, mouse_y), Mat2::from_diagonal(vec2(64.0, 64.0)), sprite(3, 1), mq::WHITE);
        }
    }





    {
        let tl = vec2(0.0, 0.0) + view_offset;
        let sz = world_size_f32 * view_scale;
        mq::draw_rectangle_lines(tl.x, tl.y, sz.x, sz.y, 2.0, mq::RED);
    }
}

pub fn on_screen_mat(pos: Vec2, tf: Mat2) -> bool {
    let aabb = vec2(f32::max(tf.x_axis.x.abs(), tf.y_axis.x.abs()),
                    f32::max(tf.x_axis.y.abs(), tf.y_axis.y.abs()));

    return on_screen(pos, aabb);
}

pub fn on_screen(pos: Vec2, aabb: Vec2) -> bool {

    if pos.x + aabb.x <= 0.0 { return false; }
    if pos.y + aabb.y <= 0.0 { return false; }
    if pos.x - aabb.x >= mq::screen_width()  { return false; }
    if pos.y - aabb.y >= mq::screen_height() { return false; }

    return true;
}

pub fn draw_texture_gwah_checked(texture: &mq::Texture2D, pos: Vec2, tf: Mat2, coord: (Vec2,Vec2), color: mq::Color) {

    if on_screen_mat(pos, tf) { draw_texture_gwah(&texture, pos, tf, coord, color); }
}


pub fn draw_texture_gwah(texture: &mq::Texture2D, pos: Vec2, tf: Mat2, coord: (Vec2,Vec2), color: mq::Color) {

    let vpos = [
        pos + tf.mul_vec2(vec2(-0.5, -0.5)),
        pos + tf.mul_vec2(vec2( 0.5, -0.5)),
        pos + tf.mul_vec2(vec2( 0.5,  0.5)),
        pos + tf.mul_vec2(vec2(-0.5,  0.5))
    ];

    let vrtx = [
        mq::Vertex::new(vpos[0].x.floor(), vpos[0].y.floor(), 0.0, coord.0.x, coord.0.y, color),
        mq::Vertex::new(vpos[1].x.floor(), vpos[1].y.floor(), 0.0, coord.1.x, coord.0.y, color),
        mq::Vertex::new(vpos[2].x.floor(), vpos[2].y.floor(), 0.0, coord.1.x, coord.1.y, color),
        mq::Vertex::new(vpos[3].x.floor(), vpos[3].y.floor(), 0.0, coord.0.x, coord.1.y, color),
    ];

    let indx: [u16; 6] = [0, 1, 2, 0, 2, 3];

    let gl = unsafe { mq::get_internal_gl().quad_gl };

    gl.draw_mode(mq::DrawMode::Triangles);
    gl.texture(Some(texture));
    gl.geometry(&vrtx, &indx);
}

