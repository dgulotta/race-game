use notan::{
    app::Graphics,
    draw::CreateDraw,
    math::{Affine2, Mat2, Vec2},
};

use crate::{course::bounding_rect, states::RaceState};

use super::{
    graphics::{TILE_SIZE, TileGraphics},
    loader::Resources,
    race::{anh, gfx_size_for},
};

pub(super) fn make_animation_webp(
    gfx: &mut Graphics,
    res: &Resources,
    state: &RaceState,
    zoom: f32,
    bg_color: &[f32; 3],
) -> Result<Vec<u8>, anyhow::Error> {
    let course = state.sim.get_course();
    let (xrange, yrange) = bounding_rect(course.keys());
    let tile_size_zoom = TILE_SIZE * zoom;
    let width = gfx_size_for(xrange.end() - xrange.start() + 1, zoom) & !0x1;
    let height = gfx_size_for(yrange.end() - yrange.start() + 1, zoom) & !0x1;
    let texture = gfx
        .create_render_texture(width, height)
        .build()
        .map_err(anh)?;
    let mut pix = vec![0; 4 * (width as usize) * (height as usize)];
    let params = webp_animator::Params {
        width,
        height,
        background_bgra: [0xFF; 4],
        loop_count: 0,
        has_alpha: true,
    };
    let mut encoder = webp_animator::WebPAnimator::new(params)?;
    let xoff = (*xrange.start() as f32) * tile_size_zoom;
    let yoff = ((*yrange.end() + 1) as f32) * tile_size_zoom;
    let aff = Affine2::from_mat2_translation(
        Mat2::from_diagonal(Vec2::new(1.0, -1.0)),
        Vec2::new(-xoff, yoff),
    );
    let mut frame_buf = Vec::new();
    for round in 0..state.tracker.rounds_available() {
        let mut graphics = TileGraphics {
            res,
            zoom,
            bg_color,
            draw: texture.create_draw(),
            round,
        };
        graphics.draw.transform().push(aff.into());
        graphics.draw_course(course);
        for car in &state.tracker.get_cars()[round] {
            graphics.draw_car(car);
            graphics.draw_car_number(car);
        }
        gfx.render_to(&texture, &graphics.draw);
        gfx.read_pixels(&texture).read_to(&mut pix).map_err(anh)?;
        frame_buf.clear();
        let frame_enc = image_webp::WebPEncoder::new(&mut frame_buf);
        frame_enc.encode(&pix, width, height, image_webp::ColorType::Rgba8)?;
        encoder.add_webp_image(&frame_buf, None, 500)?;
    }
    let mut out = Vec::new();
    encoder.write(&mut out)?;
    Ok(out)
}

/*
pub(super) fn make_animation_png(
    gfx: &mut Graphics,
    res: &Resources,
    state: &RaceState,
    zoom: f32,
    bg_color: &[f32; 3],
) -> Result<Vec<u8>, anyhow::Error> {
    let mut out = Cursor::new(Vec::new());
    let course = state.sim.get_course();
    let (xrange, yrange) = bounding_rect(course.keys());
    let tile_size_zoom = TILE_SIZE * zoom;
    let width = gfx_size_for(xrange.end() - xrange.start() + 1, zoom);
    let height = gfx_size_for(yrange.end() - yrange.start() + 1, zoom);
    let texture = gfx
        .create_render_texture(width, height)
        .build()
        .map_err(anh)?;
    let mut pix = vec![0; 4 * (width as usize) * (height as usize)];
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder
        .set_animated(state.tracker.rounds_available() as u32, 0)
        .unwrap();
    let mut writer = encoder.write_header().unwrap();
    writer.set_frame_delay(1, 2).unwrap();
    let xoff = (*xrange.start() as f32) * tile_size_zoom;
    let yoff = ((*yrange.end() + 1) as f32) * tile_size_zoom;
    let aff = Affine2::from_mat2_translation(
        Mat2::from_diagonal(Vec2::new(1.0, -1.0)),
        Vec2::new(-xoff, yoff),
    );
    for round in 0..state.tracker.rounds_available() {
        let mut graphics = TileGraphics {
            res,
            zoom,
            bg_color,
            draw: texture.create_draw(),
            round,
        };
        graphics.draw.transform().push(aff.into());
        graphics.draw_course(course);
        for car in &state.tracker.get_cars()[round] {
            graphics.draw_car(car);
            graphics.draw_car_number(car);
        }
        gfx.render_to(&texture, &graphics.draw);
        gfx.read_pixels(&texture).read_to(&mut pix).map_err(anh)?;
        writer.write_image_data(&pix).unwrap();
    }
    writer.finish().unwrap();
    Ok(out.into_inner())
}
*/
