pub enum SMDtype {
    Connect,
    Disconnect,
    Updatequest,
    Update,
    ToUpload,
    ToDownload,
    Other,
}

impl SMDtype {
    pub fn to_value(&self) -> u8 {
        match *self {
            Self::Connect => 1,
            Self::Disconnect => 2,
            Self::ToUpload => 3,
            Self::ToDownload => 4,
            _ => 0,
        }
    }

    pub fn from_value(value: u8) -> Self {
        match value {
            1 => Self::Connect,
            2 => Self::Disconnect,
            3 => Self::ToUpload,
            4 => Self::ToDownload,
            _ => Self::Other
        }
    }
}
