//! Validation of loaded model invariants before replacement.
use crate::workbench::{
    catalog::Kind,
    session::{Model, Session},
};
mod symbolic;
pub(super) fn session(s: &Session) -> Result<(), String> {
    s.settings.validate(s.kind)?;
    if s.input.chars().count() > 256
        || !s.elapsed.is_finite()
        || !(0.0..=1.0).contains(&s.elapsed)
        || s.ticks > u64::MAX - 10000
    {
        return Err("Invalid input, clock or tick counter".into());
    }
    let valid = match (&s.model, s.kind) {
        (Model::Grid(g), Kind::Rule30 | Kind::Rule110 | Kind::Life | Kind::Hpp) => {
            let size = s.settings.world.size(s.kind);
            g.kind == s.kind
                && g.grid.size == size
                && g.boundary == s.settings.world.boundary
                && g.grid.voxels.len() == (size.x * size.y) as usize
                && g.history.len() <= 64
                && g.history.iter().all(|r| r.len() == size.x as usize)
        }
        (Model::Boids(f), Kind::Boids) => {
            f.boids.len() == s.settings.agent_count
                && f.boids.iter().all(|b| {
                    b.position.is_finite()
                        && b.velocity.is_finite()
                        && b.position.cmpge(glam::Vec2::ZERO).all()
                        && b.position.cmplt(glam::Vec2::splat(64.0)).all()
                        && b.velocity.length() <= 5.001
                })
        }
        (Model::Ants(a), Kind::Ants) => {
            let inside =
                |p: glam::IVec3| p.z == 0 && (0..64).contains(&p.x) && (0..64).contains(&p.y);
            a.colony.ants.len() == s.settings.agent_count
                && a.deliveries < u64::MAX - 10000
                && inside(a.colony.home)
                && a.colony.food_sources.iter().all(|p| inside(*p))
                && a.colony.ants.iter().all(|ant| inside(ant.pos))
                && a.colony
                    .pheromone
                    .iter()
                    .all(|(p, v)| inside(*p) && v.is_finite() && *v >= 0.0)
        }
        (Model::Symbolic(m), _) => symbolic::valid(m, s.kind),
        _ => false,
    };
    if !valid {
        return Err("Model state does not match its kind or arena bounds".into());
    }
    Ok(())
}
