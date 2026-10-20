//! Quantitative comparisons use complete fields with explicit dimensions and state units.
#[derive(Clone, Debug)]
pub struct Field {
    pub size: [u32; 3],
    pub units: &'static str,
    pub values: Vec<f32>,
}
#[derive(Clone, Debug)]
pub struct Difference {
    pub mean_absolute: f64,
    pub root_mean_square: f64,
    pub maximum: f32,
    pub changed: usize,
    pub projection: Vec<f32>,
}
impl Field {
    pub fn compare(&self, other: &Self) -> Result<Difference, String> {
        let count = self
            .size
            .iter()
            .try_fold(1usize, |n, v| n.checked_mul(*v as usize));
        if self.size != other.size || self.units != other.units {
            return Err("Field differences require matching dimensions and state units".into());
        }
        if self.size.contains(&0)
            || count != Some(self.values.len())
            || self.values.len() != other.values.len()
            || self
                .values
                .iter()
                .chain(&other.values)
                .any(|v| !v.is_finite())
        {
            return Err("Invalid field dimensions or values".into());
        }
        let plane = (self.size[0] * self.size[1]) as usize;
        let mut d = Difference {
            mean_absolute: 0.0,
            root_mean_square: 0.0,
            maximum: 0.0,
            changed: 0,
            projection: vec![0.0; plane],
        };
        for (i, (a, b)) in self.values.iter().zip(&other.values).enumerate() {
            let error = (a - b).abs();
            d.mean_absolute += error as f64;
            d.root_mean_square += (error as f64).powi(2);
            d.maximum = d.maximum.max(error);
            d.changed += usize::from(error > 0.0);
            d.projection[i % plane] = d.projection[i % plane].max(error);
        }
        d.mean_absolute /= self.values.len() as f64;
        d.root_mean_square = (d.root_mean_square / self.values.len() as f64).sqrt();
        Ok(d)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_measures_and_projection_include_every_z_plane() {
        let a = Field {
            size: [2, 1, 2],
            units: "density",
            values: vec![0.0; 4],
        };
        let mut b = a.clone();
        b.values = vec![0.0, 0.5, 1.0, 0.5];
        let d = a.compare(&b).unwrap();
        assert_eq!(d.mean_absolute, 0.5);
        assert_eq!(d.changed, 3);
        assert_eq!(d.maximum, 1.0);
        assert_eq!(d.projection, vec![1.0, 0.5]);
        assert!((d.root_mean_square - (1.5_f64 / 4.0).sqrt()).abs() < 1e-12);
        b.units = "direction bits";
        assert!(a.compare(&b).is_err());
        b.units = "density";
        b.values[0] = f32::NAN;
        assert!(a.compare(&b).is_err());
    }
}
