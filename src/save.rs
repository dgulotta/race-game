use std::{
    collections::{BTreeMap, HashMap},
    hash::{DefaultHasher, Hash, Hasher},
};

use bevy_pkv::{GetError, PkvStore};
use notan::log::error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    course::{Course, TileCoord},
    level::{LevelData, SolveData},
    tile::Tile,
    ui::loader::load_levels,
};

fn hash_for<T: Hash>(data: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

fn make_pkv() -> PkvStore {
    PkvStore::new("dgulotta", "race-game")
}

pub fn load_or_log_err<T: DeserializeOwned>(key: &str, err_msg: &str) -> Option<T> {
    let pkv = make_pkv();
    load::<T>(&pkv, key).unwrap_or_else(|e| {
        error!("{err_msg}: {e}");
        None
    })
}

pub fn save_or_log_err<T: Serialize>(key: &str, value: &T, err_msg: &str) {
    let mut pkv = make_pkv();
    if let Err(e) = save(&mut pkv, key, &value) {
        error!("{err_msg}: {e}");
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load<T: DeserializeOwned>(pkv: &PkvStore, key: &str) -> Result<Option<T>, String> {
    match pkv.get(key) {
        Ok(t) => Ok(Some(t)),
        Err(GetError::NotFound) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn load<T: DeserializeOwned>(pkv: &PkvStore, key: &str) -> Result<Option<T>, String> {
    use base64::engine::{general_purpose::STANDARD, Engine};
    let data_str: String = match pkv.get(key) {
        Ok(s) => s,
        Err(GetError::NotFound) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let data = STANDARD.decode(data_str).map_err(|err| err.to_string())?;
    rmp_serde::decode::from_slice(&data).map_err(|err| err.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save<T: Serialize>(pkv: &mut PkvStore, key: &str, value: &T) -> Result<(), String> {
    pkv.set(key, value).map_err(|e| e.to_string())
}

#[cfg(target_arch = "wasm32")]
pub fn save<T: Serialize>(pkv: &mut PkvStore, key: &str, value: &T) -> Result<(), String> {
    use base64::engine::{general_purpose::STANDARD, Engine};
    let data = rmp_serde::encode::to_vec(value).map_err(|err| err.to_string())?;
    let data_str = STANDARD.encode(data);
    pkv.set(key, &data_str).map_err(|err| err.to_string())
}

pub fn save_course(lvl: &LevelData, course: &Course) {
    let key = format!("track/{}", hash_for(lvl));
    save_or_log_err(&key, course, "Failed to save course")
}

/*
pub fn have_saved_course(lvl: &LevelData) -> bool {
    let key = format!("track/{}", hash_for(lvl));
    let pkv = make_pkv();
    !matches!(pkv.get::<()>(&key), Err(GetError::NotFound))
}
*/

pub fn load_course(lvl: &LevelData) -> Option<Course> {
    let key = format!("track/{}", hash_for(lvl));
    load_or_log_err(&key, "Failed to load course")
    //let data: Vec<(TileCoord, Tile)> = load_or_log_err(&key, "Failed to load course")?;
    //Some(data.into_iter().collect())
}

pub fn course_is_nonempty(lvl: &LevelData) -> bool {
    load_course(lvl).is_some_and(|c| !c.is_empty())
}

pub fn save_solve(lvl: &LevelData, solve: &SolveData) {
    let key = format!("solve/{}", hash_for(lvl));
    let best = solve.combine_option(&load_or_log_err(&key, "Failed to load solve data"));
    save_or_log_err(&key, &best, "Failed to save solve data");
}

pub fn load_solve(lvl: &LevelData) -> Option<SolveData> {
    let key = format!("solve/{}", hash_for(lvl));
    load_or_log_err(&key, "Failed to load solve data")
}

#[derive(Serialize, Deserialize)]
pub struct TileData {
    pub coord: TileCoord,
    #[serde(flatten)]
    pub tile: Tile,
}

pub fn course_to_vec(course: &Course) -> Vec<TileData> {
    course
        .iter()
        .map(|(k, v)| TileData {
            coord: *k,
            tile: *v,
        })
        .collect()
}

pub fn saved_courses_to_toml() -> String {
    let levels = load_levels();
    let data: BTreeMap<_, _> = levels
        .iter()
        .filter_map(|lvl| {
            let course = course_to_vec(&load_course(lvl)?);
            /*
            let course: Vec<_> = load_course(lvl)?
                .iter()
                .map(|(k, v)| TileData {
                    coord: *k,
                    tile: *v,
                })
                .collect();
            */
            Some((lvl.name.clone(), course))
        })
        .collect();
    toml::to_string(&data).unwrap()
}

pub fn courses_from_toml(data: &str) -> Result<HashMap<String, Course>, toml::de::Error> {
    let data: HashMap<String, Vec<TileData>> = toml::from_str(data)?;
    let all = data
        .into_iter()
        .map(|(k, v)| {
            let c: Course = v.iter().map(|d| (d.coord, d.tile)).collect();
            (k, c)
        })
        .collect();
    Ok(all)
}
