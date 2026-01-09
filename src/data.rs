#[derive(Debug, Clone)]
pub struct DataStore {
    pub items_loaded: usize,
    pub monsters_loaded: usize,
}

impl DataStore {
    pub fn empty() -> Self {
        Self {
            items_loaded: 0,
            monsters_loaded: 0,
        }
    }
}
