use strum::IntoEnumIterator;

use crate::{
    course::TileCoord,
    direction::{DihedralElement, Direction, ROTATIONS, rotation_for},
    path::track_tile,
    save::courses_from_toml,
    simulator::{CarCoord, Simulator, is_entrance, is_exit},
    tile::{Tile, TileType},
    tracker::Tracker,
    ui::loader::load_levels,
};

#[test]
fn test_inverses() {
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
fn test_track_tile() {
    let pos = TileCoord(0, 0);
    let cpos: CarCoord = pos.into();
    let straight_start = Direction::Down;
    let straight_end = Direction::Up;
    let turn_start = Direction::Down;
    let turn_end = Direction::Left;
    assert!(is_entrance(TileType::Straight, straight_start.opposite()));
    assert!(is_exit(TileType::Straight, straight_end));
    assert!(is_entrance(TileType::Turn, turn_start.opposite()));
    assert!(is_exit(TileType::Turn, turn_end));
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
