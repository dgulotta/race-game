use strum::IntoEnumIterator;

use crate::{
    combine::combine_options,
    course::TileCoord,
    direction::{
        DihedralElement, Direction, ROTATIONS, reflection_along, rotation_for, trans_for_dirs,
    },
    path::track_tile,
    save::courses_from_toml,
    simulator::{
        CarCoord, STRAIGHT_ENTRANCE, STRAIGHT_EXIT, Simulator, TURN_ENTRANCE, TURN_EXIT,
        is_entrance, is_entrance_id, is_exit, is_exit_id,
    },
    tile::{Tile, TileType},
    tracker::Tracker,
    ui::loader::load_levels,
};

#[test]
fn test_inverses() {
    for d in Direction::iter() {
        assert_eq!(d, d.opposite().opposite());
        assert_eq!(d.opposite(), DihedralElement::Rot180 * d);
    }
    for e in DihedralElement::iter() {
        assert_eq!(e * e.inverse(), DihedralElement::Id);
        assert_eq!(e.inverse() * e, DihedralElement::Id);
        for d in Direction::iter() {
            assert_eq!(e.apply_inverse(d), e.inverse() * d);
        }
    }
}

#[test]
fn test_associativity() {
    for e1 in DihedralElement::iter() {
        for e2 in DihedralElement::iter() {
            for d in Direction::iter() {
                assert_eq!((e1 * e2) * d, e1 * (e2 * d));
            }
            for e3 in DihedralElement::iter() {
                assert_eq!((e1 * e2) * e3, e1 * (e2 * e3));
            }
        }
    }
}

#[test]
fn test_identity() {
    for e in DihedralElement::iter() {
        assert_eq!(DihedralElement::Id * e, e);
        assert_eq!(e * DihedralElement::Id, e);
    }
    for d in Direction::iter() {
        assert_eq!(DihedralElement::Id * d, d);
    }
}

#[test]
fn test_rotation_for() {
    for d1 in Direction::iter() {
        for d2 in Direction::iter() {
            let e = rotation_for(d1, d2);
            assert_eq!(e.sign(), 1);
            assert_eq!(e * d1, d2);
        }
        for e in ROTATIONS {
            assert_eq!(e, rotation_for(d1, e * d1));
        }
    }
}

#[test]
fn test_reflect_along() {
    for d in Direction::iter() {
        let e = reflection_along(d);
        assert_eq!(e.sign(), -1);
        assert_eq!(d, e * d);
    }
}

#[test]
fn test_trans_for_dirs() {
    for from1 in Direction::iter() {
        for r in [DihedralElement::Rot90, DihedralElement::Rot270] {
            let from2 = r * from1;
            for e in DihedralElement::iter() {
                let f = trans_for_dirs(from1, from2, e * from1, e * from2);
                assert_eq!(e, f);
            }
        }
    }
}

#[test]
fn test_track_tile() {
    let pos = TileCoord(0, 0);
    let cpos: CarCoord = pos.into();
    let straight_start = STRAIGHT_ENTRANCE.opposite();
    let straight_end = STRAIGHT_EXIT;
    let turn_start = TURN_ENTRANCE.opposite();
    let turn_end = TURN_EXIT;
    assert!(is_entrance_id(
        TileType::Straight,
        straight_start.opposite()
    ));
    assert!(is_exit_id(TileType::Straight, straight_end));
    assert!(is_entrance_id(TileType::Turn, turn_start.opposite()));
    assert!(is_exit_id(TileType::Turn, turn_end));
    for r in ROTATIONS {
        let p1 = cpos + r * straight_start;
        let p2 = cpos + r * straight_end;
        assert_eq!(
            track_tile(p1, p2),
            (
                pos,
                Tile {
                    tile_type: TileType::Straight,
                    transform: r,
                    offset: 0
                }
            )
        );
    }
    for e in DihedralElement::iter() {
        let p1 = cpos + e * turn_start;
        let p2 = cpos + e * turn_end;
        assert_eq!(
            track_tile(p1, p2),
            (
                pos,
                Tile {
                    tile_type: TileType::Turn,
                    transform: e,
                    offset: 0
                }
            )
        );
    }
}

#[test]
fn test_solutions() {
    let solutions = courses_from_toml(include_str!("../res/solutions.toml")).unwrap();
    for lvl in load_levels() {
        if let Some(course) = solutions.get(&lvl.name) {
            for tile in course.values() {
                assert!(!lvl.banned[tile.tile_type]);
            }
            let mut tracker = Tracker::new(lvl.cars);
            let mut sim = Simulator::new(course.clone(), lvl.cars);
            while !(sim.is_finished() || tracker.is_loop_detected()) {
                sim.run_round();
                for ev in sim.events() {
                    tracker.process_event(ev);
                }
            }
            assert_eq!(&lvl.finish, tracker.get_finishes());
        }
    }
}

#[test]
fn test_entrance_exit() {
    for t in TileType::iter() {
        for d in Direction::iter() {
            assert!(!(is_entrance_id(t, d.opposite()) && is_exit_id(t, d)));
        }
    }
}

#[test]
fn test_combine() {
    for tile1 in [TileType::Straight, TileType::Turn] {
        for tile2 in TileType::iter() {
            for tr1 in DihedralElement::iter() {
                for tr2 in DihedralElement::iter() {
                    let orig = Tile {
                        tile_type: tile2,
                        transform: tr2,
                        offset: 0,
                    };
                    let add = Tile {
                        tile_type: tile1,
                        transform: tr1,
                        offset: 0,
                    };
                    for combined in combine_options(orig, add) {
                        for d in Direction::iter() {
                            assert!(
                                is_entrance(combined, d)
                                    == (is_entrance(orig, d) && !is_exit(add, d.opposite()))
                                        | is_entrance(add, d)
                            );
                            assert!(
                                is_exit(combined, d)
                                    == (is_exit(orig, d) && !is_entrance(add, d.opposite()))
                                        | is_exit(add, d)
                            );
                        }
                    }
                }
            }
        }
    }
}
