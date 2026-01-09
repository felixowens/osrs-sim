#[derive(Debug, Clone, Copy)]
pub struct Levels {
    pub attack: u8,
    pub strength: u8,
    pub defence: u8,
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub levels: Levels,
}

#[derive(Debug, Clone)]
pub struct Build {
    pub weapon_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
}

pub struct EvalContext<'a> {
    pub player: &'a PlayerState,
    pub build: &'a Build,
    pub target: &'a Target,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EvalResult {
    pub dps: f64,
    pub max_hit: u32,
    pub accuracy: f64,
    pub interval_ticks: u8,
}
