//! Two rule layers share one world and one clock; their combined output feeds back.
use super::Rule;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Blend {
    #[default]
    Union,
    Intersection,
    ExclusiveOr,
}
impl Blend {
    pub fn combine(self, left: bool, right: bool) -> bool {
        match self {
            Self::Union => left || right,
            Self::Intersection => left && right,
            Self::ExclusiveOr => left ^ right,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub rule: Rule,
    pub blend: Blend,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub primary: Rule,
    pub secondary: Option<Layer>,
}
impl Program {
    pub fn validate(&self) -> Result<(), &'static str> {
        self.primary.validate()?;
        if let Some(layer) = &self.secondary {
            layer.rule.validate()?;
        }
        Ok(())
    }
    pub fn evaluate(&self, neighborhood: u16) -> bool {
        let left = self.primary.evaluate(neighborhood);
        self.secondary.as_ref().map_or(left, |layer| {
            layer.blend.combine(left, layer.rule.evaluate(neighborhood))
        })
    }
    pub fn title(&self) -> String {
        match &self.secondary {
            Some(layer) => format!(
                "{} · {:?} · {}",
                self.primary.name, layer.blend, layer.rule.name
            ),
            None => self.primary.name.clone(),
        }
    }
}
