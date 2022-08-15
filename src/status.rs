#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Status {
    #[default]
    Up,
    Down,
}

impl Status {
    pub fn up(&mut self) {
        *self = Self::Up;
    }

    pub fn down(&mut self, msg: &str) {
        if *self == Self::Up {
            log::warn!("{msg}");
        } else {
            log::info!("{msg}");
        }
        *self = Self::Down;
    }
}
