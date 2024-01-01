use crate::game::*;

use std::fmt::Write;
use glam::{vec2, Vec2, uvec2, UVec2, Mat2, mat2};

use std::ops::Not;



pub struct GameDraw {
    pub sprites: mq::Texture2D,
    pub font: mq::Font,
    pub stupid: String,

    pub cam_size: f32,
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
        font:               mq::load_ttf_font("tf/custom/atkinson.ttf").await.unwrap(),
        stupid:             Default::default(),
        cam_size:           10.0,
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

pub fn mach_sprite(spec: &MachineSpec) -> ((Vec2, Vec2), bool) {
    match spec {
        MachineSpec::None => panic!(),
        MachineSpec::Turret{ammo: _, can_fire_time_us: _} => (sprite(5, 2), true),
        MachineSpec::Conveyor{item: _, filter: true, can_move_time_us: _, can_dump_time_us: _} => (sprite(4, 2), true),
        MachineSpec::Conveyor{item: _, filter: false, can_move_time_us: _, can_dump_time_us: _} => (sprite(3, 2), true)
    }
}


pub fn stupid_rectangle(string: &str, pos: Vec2, center: bool, font: Option<&mq::Font>, screen_size: Vec2, view_scale: f32) {

    let font_size = (20.0*view_scale) as u16;

    let (lines, text_width) = {
        let mut text_width = 0.0_f32;
        let mut lines = 0_u32;
        for splitipy in string.split("\n") {
            text_width = text_width.max(mq::measure_text(splitipy, font, font_size, 1.0).width);
            lines += 1;
        }

        lines = lines.saturating_sub(1);
        (lines, text_width)
    };



    let width = text_width + 9.0*view_scale;
    let height = 10.0*view_scale + (lines * font_size as u32) as f32;
    let posx = f32::min(pos.x - if center {width/2.0} else {0.0}, screen_size.x - width);
    let posy = f32::min(pos.y + TILE_SIZE.y*view_scale, screen_size.y - height);

    mq::draw_rectangle(posx, posy, width, height, mq::Color::new(0.0, 0.0, 0.0, 0.75));
    mq::draw_rectangle_lines(posx, posy, width, height, 2.0, mq::WHITE);

    let mut yoffset = font_size as f32;
    for splitipy in string.split("\n") {
        mq::draw_text_ex(splitipy, posx + 4.0*view_scale, posy + 2.0*view_scale + yoffset, mq::TextParams {
            font,
            font_size,
            //font_scale: 0.5*view_scale,
            color: mq::WHITE,
            ..Default::default()
        });
        yoffset += font_size as f32;
    }
}

pub fn draw_game(main: &GameMain, draw: &mut GameDraw) {

    let view_size = TILE_SIZE.x * draw.cam_size;
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
        if let Some(feral) = main.feral_by_tile.get(&(draw.mouse_select.x as u8, draw.mouse_select.y as u8)) {
            return TileThing::Feral(feral.clone());
        } else if let Some(mach) = main.mach_by_tile.get(&(draw.mouse_select.x as u8, draw.mouse_select.y as u8)) {
            return TileThing::Machine(mach.clone());
        }
        return TileThing::None;
    })();

    // DRAW!

    mq::clear_background(mq::Color::from_hex(0x274023));

    // grid background

    let ofx = view_offset.x % (TILE_SIZE.x*view_scale*2.0);
    let ofy = view_offset.y % (TILE_SIZE.y*view_scale*2.0);

    let tile_w = (screen_size.x / (TILE_SIZE.x*view_scale)) as i32 / 2 + 3;
    let tile_h = (screen_size.y / (TILE_SIZE.y*view_scale)) as i32 + 4;


    for y in 0..tile_h {
        for x in 0..tile_w {

            let sx = TILE_SIZE.x*view_scale * ((x*2 + (y%2) - 2) as f32);
            let sy = TILE_SIZE.y*view_scale * ((y - 1) as f32);

            mq::draw_rectangle(ofx + sx, ofy +sy, TILE_SIZE.x*view_scale, TILE_SIZE.y*view_scale, mq::Color::from_hex(0x35552f));
        }
    }




    // Draw rail
    for rail in &main.rail {

        let pos = (rail.pos.as_vec2() + 0.5) * TILE_SIZE * view_scale + view_offset;

        let mat = dir_to_mat2(&rail.dir) * Mat2::from_diagonal(TILE_SIZE * view_scale);

        let coord = match rail.bend {
            Bend::Forward => sprite(0, 2),
            Bend::Right   => sprite(1, 2),
            Bend::Left    => flip_y(sprite(1, 2))
        };

        draw_texture_gwah_checked(&draw.sprites, pos, mat, coord, mq::WHITE);
    }

    // Draw machines
    for id in main.mach_ids.iter_ids() {
        let d = &main.mach_data[id.0];
        if let Some(pos) = d.pos {
            let dpos = (pos.as_vec2() + 0.5) * TILE_SIZE * view_scale + view_offset;

            if on_screen(dpos, TILE_SIZE).not() {
                continue;
            }

            let mat = Mat2::from_diagonal(TILE_SIZE * view_scale);
            let matrot = dir_to_mat2(&d.dir) * mat;

            let (ssprite, on_floor) = mach_sprite(&d.spec);

            if on_floor {
                draw_texture_gwah(&draw.sprites, dpos, matrot, ssprite, mq::WHITE);
            } else {
                draw.stupidraw.push((dpos.y, dpos, matrot, ssprite));
            }

            if let MachineSpec::Conveyor { item, filter, can_move_time_us: _, can_dump_time_us: _} = &d.spec {
                if item.count != 0 {
                    draw.stupidraw.push((dpos.y, dpos, mat, main.itemtype_data[item.itemtype.0].sprite));
                } else if *filter && item.itemtype != Default::default() {
                    let mat = Mat2::from_diagonal(TILE_SIZE * view_scale * 0.5);
                    draw_texture_gwah(&draw.sprites, dpos, mat, main.itemtype_data[item.itemtype.0].sprite, mq::Color::new(1.0, 1.0, 1.0, 0.5));
                }
            }
        }
    }

    // Draw drones
    for id in main.drone_ids.iter_ids() {

        let pos = main.drone_pos[id.0] * view_scale + view_offset;

        let mat = Mat2::from_diagonal(TILE_SIZE * view_scale);

        if on_screen_mat(pos, mat) {

            let ssprite = if ((draw.clock_1s + (id.0 as f32) * 1.618) * 4.0).fract() > 0.5 { sprite(1, 0) } else { sprite(2, 0) };

            draw.stupidraw.push((pos.y, pos, mat, ssprite));
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
            let Some(asdf) = &d.slots[idx] else { panic!() };
            let dit = &main.itemtype_data[asdf.itemtype.0];
            draw.stupidraw.push((dpos.y, dpos + offset*TILE_SIZE, mat, dit.sprite));
        };

        match count {
            1 => {
                draw_it_uwu(0, vec2(0.0, 0.0));
            },
            2 => {
                draw_it_uwu(1, vec2(-0.125, -0.0625));
                draw_it_uwu(0, vec2(0.125, 0.0625));
            },
            3 => {
                draw_it_uwu(2, vec2(0.125, -0.125));
                draw_it_uwu(1, vec2(-0.125, 0.0));
                draw_it_uwu(0, vec2(0.125, 0.125));
            },
            4 => {
                draw_it_uwu(3, vec2(-0.125, -0.1875));
                draw_it_uwu(2, vec2(0.125, -0.0625));
                draw_it_uwu(1, vec2(-0.125, 0.0625));
                draw_it_uwu(0, vec2(0.125, 0.1875));
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
                    match draw.under_cursor {
                        TileThing::None => {
                            mq::draw_rectangle_lines(select_pos.x, select_pos.y, select_size.x, select_size.y, 2.0, mq::WHITE);
                        },
                        TileThing::Feral(feral) => {
                            mq::draw_rectangle_lines(select_pos.x, select_pos.y, select_size.x, select_size.y, 8.0, mq::GREEN);

                            for exslot in main.feral_data[feral.0].slots.iter().rev() {
                                if let Some(slot) = exslot {
                                    let dit = &main.itemtype_data[slot.itemtype.0];
                                    write!(draw.stupid, "* {}× {}\n", slot.count, dit.name).unwrap();
                                }
                            }



                            if main.rail_by_tile.contains_key(&(draw.mouse_select.x as u8, draw.mouse_select.y as u8)).not() {

                                // >:)
                                if craft_item_recipe(&main.itemtype_data, &main.feral_data[feral.0].slots, 0).is_some() {
                                    write!(draw.stupid, "Press [1] to Disassemble\n").unwrap();
                                }
                                if craft_item_recipe(&main.itemtype_data, &main.feral_data[feral.0].slots, 1).is_some() {
                                    write!(draw.stupid, "Press [2] craft Bullets\n").unwrap();
                                }
                                if craft_item_recipe(&main.itemtype_data, &main.feral_data[feral.0].slots, 2).is_some() {
                                    write!(draw.stupid, "Press [3] craft Alignite Clump\n").unwrap();
                                }
                                if craft_machine_recipe(&main.itemtype_data, &main.feral_data[feral.0].slots, 5).is_some() {
                                    write!(draw.stupid, "Press [5] to craft Turret\n").unwrap();
                                }
                                if craft_machine_recipe(&main.itemtype_data, &main.feral_data[feral.0].slots, 6).is_some() {
                                    write!(draw.stupid, "Press [6] to craft Conveyor\n").unwrap();
                                }
                                if craft_machine_recipe(&main.itemtype_data, &main.feral_data[feral.0].slots, 7).is_some() {
                                    write!(draw.stupid, "Press [7] to craft Filterveyor\n").unwrap();
                                }
                                if craft_item_recipe(&main.itemtype_data, &main.feral_data[feral.0].slots, 69).is_some() {
                                    write!(draw.stupid, "Press [R] to OBFUSCATE\n").unwrap();
                                }
                            } else {
                                write!(draw.stupid, "Note: Cannot craft on rails!\n").unwrap();
                            }
                        },
                        TileThing::Machine(mach) => {

                            let d = &main.mach_data[mach.0];
                            match &d.spec {
                                MachineSpec::Turret { ammo, can_fire_time_us: _ } => {
                                    write!(draw.stupid, "Ammo: {}/69\n", ammo).unwrap();
                                },
                                MachineSpec::Conveyor { item, filter: true, can_move_time_us: _, can_dump_time_us: _ } => {
                                    if item.itemtype != Default::default() {
                                        write!(draw.stupid, "Filter: {}\nTo change, use a Conveyor to insert\n an item into the side\n", main.itemtype_data[item.itemtype.0].name).unwrap();
                                    } else {
                                         write!(draw.stupid, "Insert Item to select Item type\n").unwrap();
                                    }
                                },
                                _ => {}
                            };

                            mq::draw_rectangle_lines(select_pos.x, select_pos.y, select_size.x, select_size.y, 8.0, mq::GREEN);
                        }
                    };

                },
                Drag::Item(slot) => {
                    let dit = &main.itemtype_data[slot.itemtype.0];
                    let mat = Mat2::from_diagonal(TILE_SIZE * view_scale);
                    draw_texture_gwah(&draw.sprites, vec2(select_pos.x, select_pos.y) + 0.5*TILE_SIZE*view_scale, mat, sprite(4, 1), mq::WHITE);
                    draw_texture_gwah(&draw.sprites, vec2(mouse_x, mouse_y) - vec2(0.0, 0.5*TILE_SIZE.y), mat, dit.sprite, mq::Color::new(1.0, 1.0, 1.0, 0.75));
                },
                Drag::Machine(mach) => {
                    let d = &main.mach_data[mach.0];
                    let mat = Mat2::from_diagonal(TILE_SIZE * view_scale);
                    let mat_rot = mat * dir_to_mat2(&d.dir);
                    let (ssprite, _) = mach_sprite(&d.spec);
                    draw_texture_gwah(&draw.sprites, vec2(select_pos.x, select_pos.y) + 0.5*TILE_SIZE*view_scale, mat, sprite(4, 1), mq::WHITE);
                    draw_texture_gwah(&draw.sprites, vec2(mouse_x, mouse_y) - vec2(0.0, 0.5*TILE_SIZE.y), mat_rot, ssprite, mq::Color::new(1.0, 1.0, 1.0, 0.75));
                }
            }

                if draw.stupid.is_empty().not() {

                    stupid_rectangle(&draw.stupid, select_pos, false, Some(&draw.font), screen_size, view_scale);

                    draw.stupid.clear();
                }
        },
        ToolMode::GunPod => {

            write!(draw.stupid, "Ammo: {}/{}\n", main.player_gun_ammo, PLAYER_GUN_AMMO_MAX).unwrap();

            if let TileThing::Feral(feral) = draw.under_cursor {
                let d = &main.feral_data[feral.0];
                if slots_contains(d.slots.as_slice(), ITEM_BULLET, 1) {
                    write!(draw.stupid, "Press [R] to Reload\n").unwrap();
                    mq::draw_rectangle_lines(select_pos.x, select_pos.y, select_size.x, select_size.y, 8.0, mq::GREEN);
                }
            } else if main.player_gun_ammo == 0 {
                write!(draw.stupid, "Find some Bullets!\n").unwrap();
            }

            draw_texture_gwah(&draw.sprites, vec2(mouse_x, mouse_y), Mat2::from_diagonal(vec2(64.0, 64.0)), sprite(3, 1), mq::WHITE);

            if draw.stupid.is_empty().not() {
                stupid_rectangle(&draw.stupid, vec2(mouse_x, mouse_y), true, Some(&draw.font), screen_size, view_scale);
                draw.stupid.clear();
            }
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

