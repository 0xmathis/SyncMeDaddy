pub enum SMDtype {
    CONNECT,
    DISCONNECT,
    TO_UPLOAD,
    TO_DOWNLOAD,
    OTHER,
}

impl SMDtype {
    pub fn to_value(&self) -> u8 {
        match *self {
            Self::CONNECT => 1,
            Self::DISCONNECT => 2,
            Self::TO_UPLOAD => 3,
            Self::TO_DOWNLOAD => 4,
            _ => 0,
        }
    }

    pub fn from_value(value: u8) -> Self {
        match value {
            1 => Self::CONNECT,
            2 => Self::DISCONNECT,
            3 => Self::TO_UPLOAD,
            4 => Self::TO_DOWNLOAD,
            _ => Self::OTHER
        }
    }
}
