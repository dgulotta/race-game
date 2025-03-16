use strum::IntoEnumIterator;

use crate::{
    direction::{rotation_for, DihedralElement, Direction, ROTATIONS},
    save::courses_from_toml,
    simulator::Simulator,
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
