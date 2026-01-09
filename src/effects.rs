#[derive(Debug, Clone, Copy)]
pub enum Stage {
    PostEffectiveLevel,
    PreRolls,
    PostMaxHit,
}

#[derive(Debug, Clone)]
pub struct Effect {
    pub id: String,
    pub stage: Stage,
}
