use super::*;
use models::workbench::{catalog::Kind,session::{Model,Session}};
pub(super) fn cpu(s: &Session) -> Result<Field,String> {
    let Model::Grid(g)=&s.model else {return Err("Field comparison requires cellular or Lenia channels; all families expose metric traces.".into())};
    let gas=s.kind==Kind::Hpp;
    Ok(Field {size:g.grid.size.to_array(),units:if gas {"direction bits"} else {"density"},
        values:g.bytes().iter().map(|v|*v as f32/if gas {1.0} else {255.0}).collect()})
}
#[cfg(feature="gpu-compute")]
pub(super) fn gpu(s:engine_gpu::LeniaSnapshot) -> Field {
    Field {size:s.size,units:"density",values:s.continuous.map_or_else(||s.densities.iter().map(|v|*v as f32/255.0).collect(),|c|c.values)}
}
