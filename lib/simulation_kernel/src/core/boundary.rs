//! Explicit finite-domain sampling shared by cellular and continuous models.
use glam::{IVec3, UVec3};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Boundary {
    #[default]
    Dead,
    Periodic,
}
impl Boundary {
    pub fn coordinate(self, p: IVec3, size: UVec3) -> Option<IVec3> {
        let size = size.as_ivec3();
        match self {
            Self::Dead => (p.cmpge(IVec3::ZERO).all() && p.cmplt(size).all()).then_some(p),
            Self::Periodic => Some(IVec3::new(
                p.x.rem_euclid(size.x),
                p.y.rem_euclid(size.y),
                p.z.rem_euclid(size.z),
            )),
        }
    }
}
