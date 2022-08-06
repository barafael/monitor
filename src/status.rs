#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Status {
    #[default]
    Up,
    Down,
}

impl Status {
    pub fn up(&mut self) {
        *self = Status::Up;
    }

    pub fn down(&mut self, msg: &str) {
        if *self == Status::Up {
            log::warn!("{msg}");
        } else {
            log::info!("{msg}");
        }
        *self = Status::Down;
    }
}
