pub enum SMDtype {
    Connect,
    Disconnect,
    UpdateRequest,
    Update,
    Updated,
    Upload,
    Download,
    Other,
}

impl SMDtype {
    pub fn to_value(&self) -> u8 {
        match *self {
            Self::Connect => 1,
            Self::Disconnect => 2,
            Self::UpdateRequest => 3,
            Self::Update => 4,
            Self::Updated => 5,
            Self::Upload => 6,
            Self::Download => 7,
            _ => 0,
        }
    }

    pub fn from_value(value: u8) -> Self {
        match value {
            1 => Self::Connect,
            2 => Self::Disconnect,
            3 => Self::UpdateRequest,
            4 => Self::Update,
            5 => Self::Updated,
            6 => Self::Upload,
            7 => Self::Download,
            _ => Self::Other
        }
    }
}
