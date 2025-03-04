use notan::app::Texture;
use notan::draw::{CreateFont, Font};
use notan::egui::{EguiRegisterTexture, FontDefinitions, FontFamily};
use notan::prelude::Graphics;
use serde::Deserialize;
use std::rc::Rc;

use crate::level::LevelData;

pub type GuiImage = notan::egui::SizedTexture;

pub struct Resources {
    pub levels: Vec<Rc<LevelData>>,
    pub gui_tiles: Vec<GuiImage>,
    pub tiles: Vec<[Texture; 2]>,
    pub cars: Vec<Texture>,
    pub erase: Texture,
    pub font: Font,
    pub sample: GuiImage,
}

impl Resources {
    pub fn load_all(gfx: &mut Graphics) -> Self {
        Self {
            levels: load_levels(),
            gui_tiles: load_gui_tiles(gfx),
            tiles: load_tiles(gfx),
            erase: load_texture(gfx, include_bytes!("../../res/x.png")),
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

static TILE_IMAGES: &[[&[u8]; 2]] = &[
    [
        include_bytes!("../../res/tile01_0.png"),
        include_bytes!("../../res/tile01_0.png"),
    ],
    [
        include_bytes!("../../res/tile02_0.png"),
        include_bytes!("../../res/tile02_0.png"),
    ],
    [
        include_bytes!("../../res/tile03_0.png"),
        include_bytes!("../../res/tile03_0.png"),
    ],
    [
        include_bytes!("../../res/tile04_0.png"),
        include_bytes!("../../res/tile04_1.png"),
    ],
    [
        include_bytes!("../../res/tile05_0.png"),
        include_bytes!("../../res/tile05_0.png"),
    ],
    [
        include_bytes!("../../res/tile06_0.png"),
        include_bytes!("../../res/tile06_1.png"),
    ],
    [
        include_bytes!("../../res/tile07_0.png"),
        include_bytes!("../../res/tile07_0.png"),
    ],
    [
        include_bytes!("../../res/tile08_0.png"),
        include_bytes!("../../res/tile08_1.png"),
    ],
];

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

pub fn load_gui_tiles(gfx: &mut Graphics) -> Vec<GuiImage> {
    let mut tiles = vec![
        load_gui_texture(gfx, include_bytes!("../../res/cursor.png")),
        load_gui_texture(gfx, include_bytes!("../../res/x.png")),
    ];
    tiles.extend(TILE_IMAGES.iter().map(|i| load_gui_texture(gfx, i[0])));
    tiles
}

pub fn load_tiles(gfx: &mut Graphics) -> Vec<[Texture; 2]> {
    TILE_IMAGES
        .iter()
        .map(|i| [load_texture(gfx, i[0]), load_texture(gfx, i[1])])
        .collect()
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
