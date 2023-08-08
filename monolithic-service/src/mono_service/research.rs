use commons::*;

#[derive(Debug, Clone)]
pub struct HotelDistance {
    pub hotel_id: String,
    pub distance: f64,
}

#[derive(Debug, Clone)]
pub struct ReschService {
    pub name: String,
}

impl ReschService {
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
        })
    }
}
