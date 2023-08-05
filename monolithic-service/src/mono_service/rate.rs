use commons::*;

#[derive(Debug, Clone)]
pub struct RateService {
    name: String,
    hotel_db: Collection<Document>,
}

impl RateService {
    /// - Create a new rate service connected to the database.
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
            hotel_db: {
                let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
                mongo_client_options.app_name = Some(rate_svc::NAME.to_owned());
                let mongo_client = Client::with_options(mongo_client_options)?;
                mongo_client
                    .database(mongo_svc::DB)
                    .collection::<Document>(mongo_svc::coll::HOTEL)
            },
        })
    }
}
