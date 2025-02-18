use notan::draw::{
    Draw, DrawBuilder, DrawImages, DrawShapes, DrawTextSection, DrawTransform, Image,
};
use notan::egui::Rect;
use notan::math::{Affine2, Mat2, Mat3, Vec2};
use notan::prelude::*;

pub static TILE_SIZE: f32 = 96.0;

pub static CAR_SCALE_RATIO: f32 = TILE_SIZE / 256.0;

use super::loader::Resources;
use crate::course::{Course, TileCoord};
use crate::direction::{rotation_for, DihedralElement, Direction};
use crate::simulator::{CarCoord, CarData};
use crate::tile::Tile;

impl From<CarCoord> for Vec2 {
    fn from(value: CarCoord) -> Self {
        0.5 * TILE_SIZE * Self::new(value.0 as f32, value.1 as f32)
    }
}

impl From<TileCoord> for Vec2 {
    fn from(value: TileCoord) -> Self {
        TILE_SIZE * Self::new(value.0 as f32, value.1 as f32)
    }
}

pub fn matrix_for_dihedral(t: DihedralElement) -> Mat2 {
    let r = t * Direction::Right;
    let d = t * Direction::Down;
    Mat2::from_cols_array(&[r.dx() as f32, r.dy() as f32, d.dx() as f32, d.dy() as f32])
}

pub fn affine_for_dihedral(t: DihedralElement, width: f32, height: f32) -> Affine2 {
    let m = matrix_for_dihedral(t);
    let offset = 0.5 * Vec2::new(width, height);
    let center = 0.5 * Vec2::new(TILE_SIZE, TILE_SIZE);
    let v = center - m * offset;
    Affine2::from_mat2_translation(m, v)
}

pub fn transform_for(transform: DihedralElement, pos: TileCoord) -> Mat3 {
    let translation: Vec2 = pos.into();
    let aff = Affine2::from_translation(translation)
        * affine_for_dihedral(transform, TILE_SIZE, TILE_SIZE);
    aff.into()
}

pub fn transform_for_car(
    transform: DihedralElement,
    pos: CarCoord,
    width: f32,
    height: f32,
) -> Mat3 {
    let t1 = Affine2::from_translation(-0.5 * Vec2::new(width, height));
    let t2 = Affine2::from_mat2(matrix_for_dihedral(transform));
    let t3 = Affine2::from_translation(
        0.5 * TILE_SIZE * Vec2::new((pos.0 + 1) as f32, (pos.1 + 1) as f32),
    );
    (t3 * t2 * t1).into()
}

pub fn transform_for_text(
    transform: DihedralElement,
    pos: CarCoord,
    width: f32,
    height: f32,
) -> Mat3 {
    let t1 = Affine2::from_scale(Vec2::new(1.0, 1.0));
    let t2 = Affine2::from_translation(-0.5 * Vec2::new(width, height));
    let t3 = Affine2::from_mat2(matrix_for_dihedral(transform));
    let t4 = Affine2::from_translation(
        0.5 * TILE_SIZE * Vec2::new((pos.0 + 1) as f32, (pos.1 + 1) as f32),
    );
    (t4 * t3 * t2 * t1).into()
}

pub fn draw_tile_sprite<'a, 'b>(
    draw: &'a mut Draw,
    sprite: &'b Texture,
    trans: DihedralElement,
    pos: TileCoord,
) -> DrawBuilder<'a, Image<'b>> {
    let mut builder = draw.image(sprite);
    builder
        .size(TILE_SIZE, TILE_SIZE)
        .transform(transform_for(trans, pos));
    builder
}

pub fn draw_tile<'a, 'b>(
    draw: &'a mut Draw,
    tile: Tile,
    res: &'b Resources,
    pos: TileCoord,
    round: usize,
) -> DrawBuilder<'a, Image<'b>> {
    let total_offset = (round ^ (tile.offset as usize) ^ 1) & 1;
    draw_tile_sprite(
        draw,
        &res.tiles[tile.tile_type as usize][total_offset],
        tile.transform,
        pos,
    )
}

pub fn draw_highlights<'a>(draw: &mut Draw, tiles: impl IntoIterator<Item = &'a TileCoord>) {
    for pos in tiles {
        let screen_pos = ((pos.0 as f32) * TILE_SIZE, (pos.1 as f32) * TILE_SIZE);
        draw.rect(screen_pos, (TILE_SIZE, TILE_SIZE))
            .color(Color::TEAL);
    }
}

pub fn draw_car(draw: &mut Draw, res: &Resources, car: &CarData) {
    let trans = rotation_for(Direction::Up, car.dir);
    let sprite = &res.cars[color_for_car(car.id)];
    let width = sprite.width() * CAR_SCALE_RATIO;
    let height = sprite.height() * CAR_SCALE_RATIO;
    draw.image(sprite)
        .size(width, height)
        .transform(transform_for_car(trans, car.pos, width, height));
}

fn draw_number(draw: &mut Draw, res: &Resources, text: &str, mat: Mat3) {
    draw.text(&res.font, text)
        .transform(mat)
        .color(Color::BLACK)
        .size(TILE_SIZE * 0.4)
        .h_align_center()
        .v_align_middle();
}

pub fn draw_car_number(draw: &mut Draw, res: &Resources, car: &CarData) {
    let trans = rotation_for(Direction::Up, car.dir);
    let text = car.id.to_string();
    let m = 0.5 * matrix_for_dihedral(trans);
    let pos = 0.5
        * Vec2::new(
            TILE_SIZE * ((car.pos.0 + 1) as f32),
            TILE_SIZE * ((car.pos.1 + 1) as f32),
        );
    let aff = Affine2::from_mat2_translation(m, pos);
    let mat: Mat3 = aff.into();
    draw_number(draw, res, &text, mat);
    let bound = draw.last_text_bounds();
    draw.rect((bound.x, bound.y), (bound.width, bound.height))
        .transform(aff.into());
    draw_number(draw, res, &text, mat);
}

pub fn color_for_car(id: usize) -> usize {
    id % 5
}

pub fn draw_course(draw: &mut Draw, res: &Resources, course: &Course, round: usize) {
    draw.clear(Color::WHITE);
    for (pos, tile) in course {
        draw_tile(draw, *tile, res, *pos, round);
    }
}

pub fn set_offset(draw: &mut Draw, offset: &Vec2) {
    draw.transform().push(Mat3::from_translation(*offset));
}

pub fn get_draw_offset(center: &Vec2, rect: &Rect) -> Vec2 {
    let pix_center = TILE_SIZE * (*center + Vec2::new(0.5, 0.5));
    let rect_center = rect.center();
    Vec2::new(rect_center.x, rect_center.y) - pix_center
}
