use super::*;

#[test]
fn flock_update_is_order_independent_and_respects_speed_and_arena_bounds() {
    let mut forward = Flock::default();
    let mut backward = Flock {
        boids: forward.boids.iter().cloned().rev().collect(),
    };
    forward.step();
    backward.step();
    for (a, b) in forward.boids.iter().zip(backward.boids.iter().rev()) {
        assert!(a.position.distance(b.position) < 0.0001);
        assert!(a.velocity.distance(b.velocity) < 0.0001);
    }
    for _ in 0..500 {
        forward.step();
        for b in &forward.boids {
            assert!(b.position.is_finite() && b.velocity.is_finite());
            assert!((0.0..64.0).contains(&b.position.x));
            assert!((0.0..64.0).contains(&b.position.y));
            assert!(b.velocity.length() <= 5.0001);
        }
    }
}

#[test]
fn ant_rng_reset_replays_food_deliveries_and_bounded_trails() {
    let mut a = Foraging::default();
    let mut b = Foraging::default();
    for _ in 0..500 {
        a.step();
        b.step();
        for (x, y) in a.colony.ants.iter().zip(&b.colony.ants) {
            assert_eq!(x.pos, y.pos);
            assert_eq!(x.carrying_food, y.carrying_food);
            assert!((0..64).contains(&x.pos.x) && (0..64).contains(&x.pos.y));
        }
        assert_eq!(a.colony.pheromone, b.colony.pheromone);
        assert!(a.colony.pheromone.len() <= 4096);
    }
    assert!(a.deliveries > 0);
    assert_eq!(a.deliveries, b.deliveries);
}

#[test]
fn a_carrying_ant_returns_home_and_evaporation_removes_exhausted_trails() {
    let mut arena = Foraging::default();
    arena.colony.ants = vec![Ant {
        pos: IVec3::new(34, 32, 0),
        carrying_food: true,
    }];
    arena.colony.pheromone.insert(IVec3::ZERO, 0.0001);
    arena.step();
    assert!(!arena.colony.pheromone.contains_key(&IVec3::ZERO));
    arena.step();
    assert_eq!(arena.deliveries, 1);
    assert_eq!(arena.colony.ants[0].pos, arena.colony.home);
    assert!(!arena.colony.ants[0].carrying_food);
}
