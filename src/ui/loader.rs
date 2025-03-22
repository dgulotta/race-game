use enum_map::EnumMap;
use notan::app::Texture;
use notan::draw::{CreateFont, Font};
use notan::egui::{EguiRegisterTexture, FontDefinitions, FontFamily};
use notan::prelude::Graphics;
use serde::Deserialize;
use std::rc::Rc;

use crate::level::LevelData;
use crate::tile::TileType;

pub type GuiImage = notan::egui::SizedTexture;

pub struct TileImage {
    pub textures: [Texture; 2],
    pub gui_image: GuiImage,
}

pub struct Resources {
    pub levels: Vec<Rc<LevelData>>,
    pub tiles: EnumMap<TileType, TileImage>,
    pub erase: TileImage,
    pub select: GuiImage,
    pub cars: Vec<Texture>,
    pub font: Font,
    pub sample: GuiImage,
}

macro_rules! include_tile_anim {
    ($name: literal) => {{
        let bytes1 = include_bytes!(concat!("../../res/", $name, "_0.png"));
        let bytes2 = include_bytes!(concat!("../../res/", $name, "_1.png"));
        (bytes1, Some(bytes2))
    }};
}

macro_rules! include_tile_static {
    ($name: literal) => {{
        let bytes1 = include_bytes!(concat!("../../res/", $name, "_0.png"));
        (bytes1, None)
    }};
}

impl Resources {
    pub fn load_all(gfx: &mut Graphics) -> Self {
        Self {
            levels: load_levels(),
            tiles: load_tiles(gfx),
            erase: load_tile_from_bytes(gfx, include_tile_static!("x")),
            select: load_gui_texture(gfx, include_bytes!("../../res/cursor.png")),
            cars: load_cars(gfx),
            font: load_font(gfx),
            sample: load_gui_texture(gfx, include_bytes!("../../res/sample_track.png")),
        }
    }
}

#[derive(Deserialize)]
struct Levels {
    levels: Vec<Rc<LevelData>>,
}

pub fn load_levels() -> Vec<Rc<LevelData>> {
    let l: Levels = toml::from_str(include_str!("../../res/levels.toml")).unwrap();
    l.levels
}

static CAR_IMAGES: &[&[u8]] = &[
    include_bytes!("../../res/car_black_1.png"),
    include_bytes!("../../res/car_blue_1.png"),
    include_bytes!("../../res/car_green_1.png"),
    include_bytes!("../../res/car_red_1.png"),
    include_bytes!("../../res/car_yellow_1.png"),
];

fn load_texture(gfx: &mut Graphics, bytes: &[u8]) -> Texture {
    gfx.create_texture()
        .from_image(bytes)
        .with_premultiplied_alpha()
        .build()
        .unwrap()
}

fn load_gui_texture(gfx: &mut Graphics, bytes: &[u8]) -> GuiImage {
    let texture = load_texture(gfx, bytes);
    gfx.egui_register_texture(&texture)
}

fn load_tile(gfx: &mut Graphics, tile: TileType) -> TileImage {
    load_tile_from_bytes(gfx, load_tile_type(tile))
}

pub fn load_tiles(gfx: &mut Graphics) -> EnumMap<TileType, TileImage> {
    EnumMap::from_fn(|t| load_tile(gfx, t))
}

fn load_tile_from_bytes(gfx: &mut Graphics, bytes: (&[u8], Option<&[u8]>)) -> TileImage {
    let texture1 = load_texture(gfx, bytes.0);
    let texture2 = if let Some(b) = bytes.1 {
        load_texture(gfx, b)
    } else {
        texture1.clone()
    };
    let gui_image = gfx.egui_register_texture(&texture1);
    TileImage {
        textures: [texture1, texture2],
        gui_image,
    }
}

pub fn load_tile_type(tile: TileType) -> (&'static [u8], Option<&'static [u8]>) {
    use TileType::*;
    match tile {
        Straight => include_tile_static!("tile01"),
        Turn => include_tile_static!("tile02"),
        Finish => include_tile_static!("tile03"),
        LightIntersection => include_tile_anim!("tile04"),
        YieldIntersection => include_tile_static!("tile05"),
        LightTurns => include_tile_anim!("tile06"),
        Merge => include_tile_static!("tile07"),
        LightForwardTurn => include_tile_anim!("tile08"),
    }
}

pub fn load_cars(gfx: &mut Graphics) -> Vec<Texture> {
    CAR_IMAGES.iter().map(|i| load_texture(gfx, i)).collect()
}

pub fn load_font(gfx: &mut Graphics) -> Font {
    let fonts = FontDefinitions::default();
    let font_name = &fonts.families[&FontFamily::Proportional][0];
    let font = &fonts.font_data[font_name];
    gfx.create_font(&font.font).unwrap()
}
