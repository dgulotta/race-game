use notan::draw::{
    CreateDraw, Draw, DrawBuilder, DrawImages, DrawShapes, DrawTextSection, DrawTransform, Image,
};
use notan::egui::{self, Rect, Ui};
use notan::math::{Affine2, Mat2, Mat3, Vec2};
use notan::prelude::*;

pub static TILE_SIZE: f32 = 96.0;

pub static CAR_SCALE_RATIO: f32 = TILE_SIZE / 256.0;

use super::loader::Resources;
use crate::course::{Course, TileCoord};
use crate::direction::{DihedralElement, Direction, rotation_for};
use crate::playback::{CarAnimation, CarPosF};
use crate::simulator::{CarCoord, CarData};
use crate::tile::Tile;

pub struct TileGraphics<'a> {
    pub res: &'a Resources,
    pub zoom: f32,
    pub draw: Draw,
    pub round: usize,
}

impl TileGraphics<'_> {
    pub fn tile_size(&self) -> f32 {
        self.zoom * TILE_SIZE
    }

    fn car_to_screen(&self, value: CarCoord) -> Vec2 {
        0.5 * self.tile_size() * Vec2::new((value.0 + 1) as f32, (value.1 + 1) as f32)
    }

    fn car_to_screen_smooth(&self, pos: Vec2) -> Vec2 {
        0.5 * self.tile_size() * Vec2::new(pos.x + 1.0, pos.y + 1.0)
    }

    fn tile_ul_to_screen(&self, value: TileCoord) -> Vec2 {
        self.tile_size() * Vec2::new(value.0 as f32, value.1 as f32)
    }

    pub fn transform_for(&self, transform: DihedralElement, pos: TileCoord) -> Mat3 {
        let m = matrix_for_dihedral(transform);
        let translation: Vec2 = self.tile_ul_to_screen(pos);
        let hsz = 0.5 * self.tile_size();
        let mid = Vec2::new(hsz, hsz);
        let v = translation + mid - m * mid;
        let aff = Affine2::from_mat2_translation(m, v);
        aff.into()
    }

    pub fn transform_for_car(
        &self,
        transform: DihedralElement,
        pos: CarCoord,
        width: f32,
        height: f32,
    ) -> Mat3 {
        let translation = self.car_to_screen(pos);
        let offset = -0.5 * Vec2::new(width, height);
        let rot = matrix_for_dihedral(transform);
        Affine2::from_mat2_translation(rot, translation + rot * offset).into()
    }

    fn rotation_smooth(dir: Vec2) -> Mat2 {
        Mat2::from_cols(Vec2::new(-dir.y, dir.x), -dir)
    }

    pub fn transform_for_car_smooth(&self, pos: &CarPosF, width: f32, height: f32) -> Mat3 {
        let translation = self.car_to_screen_smooth(pos.pos);
        let offset = -0.5 * Vec2::new(width, height);
        let rot = Self::rotation_smooth(pos.dir);
        Affine2::from_mat2_translation(rot, translation + rot * offset).into()
    }

    pub fn draw_tile(&mut self, tile: Tile, pos: TileCoord) -> DrawBuilder<'_, Image<'_>> {
        let total_offset = (self.round ^ (tile.offset as usize) ^ 1) & 1;
        let sprite = &self.res.tiles[tile.tile_type].textures[total_offset];
        self.draw_tile_sprite(sprite, tile.transform, pos)
    }

    pub fn draw_tile_sprite<'a>(
        &mut self,
        sprite: &'a Texture,
        trans: DihedralElement,
        pos: TileCoord,
    ) -> DrawBuilder<'_, Image<'a>> {
        let tsz = self.tile_size();
        let trans = self.transform_for(trans, pos);
        let mut builder = self.draw.image(sprite);
        builder.size(tsz, tsz).transform(trans);
        builder
    }

    pub fn draw_course(&mut self, course: &Course) {
        self.draw.clear(Color::WHITE);
        for (pos, tile) in course {
            self.draw_tile(*tile, *pos);
        }
    }

    pub fn set_offset(&mut self, offset: &Vec2) {
        self.draw.transform().push(Mat3::from_translation(*offset));
    }

    pub fn draw_highlights<'a>(&mut self, tiles: impl IntoIterator<Item = &'a TileCoord>) {
        let tsz = self.tile_size();
        for pos in tiles {
            let screen_pos = ((pos.0 as f32) * tsz, (pos.1 as f32) * tsz);
            self.draw.rect(screen_pos, (tsz, tsz)).color(Color::TEAL);
        }
    }

    pub fn draw_car(&mut self, car: &CarData) {
        let rot = rotation_for(Direction::Up, car.dir);
        let sprite = &self.res.cars[color_for_car(car.id)];
        let width = sprite.width() * CAR_SCALE_RATIO * self.zoom;
        let height = sprite.height() * CAR_SCALE_RATIO * self.zoom;
        let trans = self.transform_for_car(rot, car.pos, width, height);
        self.draw.image(sprite).size(width, height).transform(trans);
    }

    pub fn draw_car_smooth(&mut self, id: usize, pos: &CarPosF) {
        let sprite = &self.res.cars[color_for_car(id)];
        let width = sprite.width() * CAR_SCALE_RATIO * self.zoom;
        let height = sprite.height() * CAR_SCALE_RATIO * self.zoom;
        let trans = self.transform_for_car_smooth(pos, width, height);
        self.draw.image(sprite).size(width, height).transform(trans);
    }

    pub fn draw_cars(&mut self, cars: &[CarData], animations: &[CarAnimation], t: f32) {
        if !animations.is_empty() {
            for anim in animations {
                self.draw_car_smooth(anim.id, &anim.position_at_time(t));
            }
        } else {
            for car in cars {
                self.draw_car(car);
                self.draw_car_number(car);
            }
        }
    }

    fn draw_number(&mut self, text: &str, mat: Mat3) {
        let size = self.tile_size() * 0.4;
        self.draw
            .text(&self.res.font, text)
            .transform(mat)
            .color(Color::BLACK)
            .size(size)
            .h_align_center()
            .v_align_middle();
    }

    fn draw_car_number_base(&mut self, id: usize, mat: Mat3) {
        let text = id.to_string();
        self.draw_number(&text, mat);
        let bound = self.draw.last_text_bounds();
        self.draw
            .rect((bound.x, bound.y), (bound.width, bound.height))
            .transform(mat);
        self.draw_number(&text, mat);
    }

    pub fn draw_car_number(&mut self, car: &CarData) {
        let trans = rotation_for(Direction::Up, car.dir);
        let m = 0.5 * matrix_for_dihedral(trans);
        let pos = self.car_to_screen(car.pos);
        let aff = Affine2::from_mat2_translation(m, pos);
        let mat: Mat3 = aff.into();
        self.draw_car_number_base(car.id, mat);
    }

    pub fn draw_car_number_smooth(&mut self, id: usize, pos: &CarPosF) {
        let m = 0.5 * Self::rotation_smooth(pos.dir);
        let pos = self.car_to_screen_smooth(pos.pos);
        let aff = Affine2::from_mat2_translation(m, pos);
        let mat: Mat3 = aff.into();
        self.draw_car_number_base(id, mat);
    }

    pub fn draw_tile_boundary(&mut self, pos: CarCoord) {
        let d = if pos.0 & 1 == 0 {
            Direction::Left
        } else {
            Direction::Up
        };
        let p1 = self.car_to_screen(pos + d);
        let p2 = self.car_to_screen(pos + d.opposite());
        self.draw
            .line((p1.x, p1.y), (p2.x, p2.y))
            .color(Color::BLACK);
    }

    fn draw_tile_boundary_arrow_car_coord(&mut self, pos: CarCoord, adj: CarCoord) {
        let base = self.car_to_screen(pos);
        let out = self.car_to_screen(adj) - base;
        let rot = Vec2::new(-out.y, out.x);
        let t1 = base - 0.2 * out;
        let t2 = base - 0.05 * out + 0.25 * rot;
        let t3 = base - 0.05 * out - 0.25 * rot;
        self.draw
            .triangle((t1.x, t1.y), (t2.x, t2.y), (t3.x, t3.y))
            .color(Color::BLACK);
    }

    pub fn draw_tile_boundary_arrow(&mut self, pos: CarCoord, adj: TileCoord) {
        self.draw_tile_boundary(pos);
        self.draw_tile_boundary_arrow_car_coord(pos, adj.into())
    }

    pub fn draw_tile_boundary_arrows(&mut self, pos: CarCoord) {
        self.draw_tile_boundary(pos);
        let d = if pos.0 & 1 == 0 {
            Direction::Up
        } else {
            Direction::Left
        };
        self.draw_tile_boundary_arrow_car_coord(pos, pos + d);
        self.draw_tile_boundary_arrow_car_coord(pos, pos + d.opposite());
    }
}

impl From<CarCoord> for Vec2 {
    fn from(value: CarCoord) -> Self {
        0.5 * TILE_SIZE * Self::new(value.0 as f32, value.1 as f32)
    }
}

pub fn matrix_for_dihedral(t: DihedralElement) -> Mat2 {
    let r = t * Direction::Right;
    let d = t * Direction::Down;
    Mat2::from_cols_array(&[r.dx() as f32, r.dy() as f32, d.dx() as f32, d.dy() as f32])
}

pub fn color_for_car(id: usize) -> usize {
    id % 5
}

pub fn get_draw_offset(center: &Vec2, rect: &Rect) -> Vec2 {
    let pix_center = TILE_SIZE * (*center + Vec2::new(0.5, 0.5));
    let rect_center = rect.center();
    Vec2::new(rect_center.x, rect_center.y) - pix_center
}

pub fn allocate_ui_space(ui: &mut Ui, zoom: f32, width: u32, height: u32) -> Rect {
    let factor = TILE_SIZE * zoom / ui.ctx().zoom_factor();
    let w = (width as f32) * factor;
    let h = (height as f32) * factor;
    let resp = ui.allocate_response(egui::Vec2::new(w, h), egui::Sense::hover());
    resp.rect * ui.ctx().zoom_factor()
}

pub fn create_draw_masked(gfx: &mut Graphics, rect: &Rect) -> Draw {
    let mut mask = gfx.create_draw();
    mask.rect((rect.min.x, rect.min.y), (rect.width(), rect.height()));
    let mut draw = gfx.create_draw();
    draw.transform()
        .push(Mat3::from_translation(Vec2::new(rect.min.x, rect.min.y)));
    draw.mask(Some(&mask));
    draw.rect((0.0, 0.0), (rect.width(), rect.height()));
    draw
}
