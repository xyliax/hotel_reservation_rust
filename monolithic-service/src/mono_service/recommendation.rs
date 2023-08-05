use commons::*;

#[derive(Debug, Clone)]
pub struct RecommService {
    name: String,
}

impl RecommService {
    /// - Create a new recommendation service.
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
        })
    }
}
