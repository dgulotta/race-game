use enum_map::EnumMap;

use crate::{
    direction::{DihedralElement, Direction, trans_for_dirs},
    simulator::{
        STRAIGHT_ENTRANCE, STRAIGHT_EXIT, TURN_ENTRANCE, TURN_EXIT, is_entrance_id, is_exit_id,
    },
    tile::{Tile, TileType},
};

enum EnterExit {
    Entrance,
    Exit,
    Neither,
}

fn check_entrance_exit(t: TileType, d: Direction) -> EnterExit {
    if is_entrance_id(t, d.opposite()) {
        EnterExit::Entrance
    } else if is_exit_id(t, d) {
        EnterExit::Exit
    } else {
        EnterExit::Neither
    }
}

fn yield_intersection_trans(dir_no_yield: Direction, dir_yield: Direction) -> DihedralElement {
    trans_for_dirs(Direction::Left, Direction::Up, dir_no_yield, dir_yield)
}

fn intersections(dir_no_yield: Direction, dir_yield: Direction) -> Vec<Tile> {
    let transform = yield_intersection_trans(dir_no_yield, dir_yield);
    vec![
        Tile {
            tile_type: TileType::YieldIntersection,
            transform,
            offset: 0,
        },
        Tile {
            tile_type: TileType::LightIntersection,
            transform,
            offset: 0,
        },
    ]
}

fn merge(dir_straight: Direction, dir_merge: Direction) -> Vec<Tile> {
    vec![Tile {
        tile_type: TileType::Merge,
        transform: trans_for_dirs(Direction::Up, Direction::Left, dir_straight, dir_merge),
        offset: 0,
    }]
}

fn light_forward_turn(dir_straight: Direction, dir_turn: Direction) -> Vec<Tile> {
    vec![Tile {
        tile_type: TileType::LightForwardTurn,
        transform: trans_for_dirs(Direction::Up, Direction::Right, dir_straight, dir_turn),
        offset: 0,
    }]
}

pub fn combine(orig: Tile, add: Tile, banned: &EnumMap<TileType, bool>) -> Tile {
    let options = combine_options(orig, add);
    for opt in options {
        if !banned[opt.tile_type] {
            return opt;
        }
    }
    add
}

pub(crate) fn combine_options(orig: Tile, add: Tile) -> Vec<Tile> {
    use EnterExit::*;
    let trans = orig.transform.inverse() * add.transform;
    match add.tile_type {
        TileType::Straight => {
            if is_entrance_id(orig.tile_type, trans * STRAIGHT_ENTRANCE)
                && is_exit_id(orig.tile_type, trans * STRAIGHT_EXIT)
            {
                vec![orig]
            } else {
                let l = check_entrance_exit(orig.tile_type, trans * Direction::Left);
                let r = check_entrance_exit(orig.tile_type, trans * Direction::Right);
                match (l, r) {
                    (Entrance, Exit) => intersections(
                        add.transform * Direction::Up,
                        add.transform * Direction::Right,
                    ),
                    (Exit, Entrance) => intersections(
                        add.transform * Direction::Up,
                        add.transform * Direction::Left,
                    ),
                    (Entrance, Neither) => merge(
                        add.transform * Direction::Up,
                        add.transform * Direction::Right,
                    ),
                    (Neither, Entrance) => merge(
                        add.transform * Direction::Up,
                        add.transform * Direction::Left,
                    ),
                    (Exit, Neither) => light_forward_turn(
                        add.transform * Direction::Up,
                        add.transform * Direction::Right,
                    ),
                    (Neither, Exit) => light_forward_turn(
                        add.transform * Direction::Up,
                        add.transform * Direction::Left,
                    ),
                    (Neither, Neither) | (Exit, Exit) => vec![],
                    (Entrance, Entrance) => unreachable!(),
                }
            }
        }
        TileType::Turn => {
            if is_entrance_id(orig.tile_type, trans * TURN_ENTRANCE)
                && is_exit_id(orig.tile_type, trans * TURN_EXIT)
            {
                vec![orig]
            } else {
                let u = check_entrance_exit(orig.tile_type, trans * Direction::Up);
                let r = check_entrance_exit(orig.tile_type, trans * Direction::Right);
                match (u, r) {
                    (Exit, Entrance) => intersections(
                        add.transform * Direction::Up,
                        add.transform * Direction::Left,
                    ),
                    (Exit, Neither) => light_forward_turn(
                        add.transform * Direction::Up,
                        add.transform * Direction::Right,
                    ),
                    (Neither, Entrance) => merge(
                        add.transform * Direction::Left,
                        add.transform * Direction::Up,
                    ),
                    (Neither, Exit) => vec![Tile {
                        tile_type: TileType::LightTurns,
                        transform: add.transform,
                        offset: 0,
                    }],
                    (Entrance, _) | (Exit, Exit) | (Neither, Neither) => vec![],
                }
            }
        }
        _ => vec![],
    }
}
