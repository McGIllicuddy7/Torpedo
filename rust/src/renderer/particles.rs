
use serde::{Deserialize, Serialize};

use crate::math::Transform;

use super::ModelData;
#[derive(Serialize, Deserialize, Clone)]
pub struct Particle{
    pub lifetime:f64,
    pub mesh:ModelData,
    pub transforms:Vec<Transform>
}
