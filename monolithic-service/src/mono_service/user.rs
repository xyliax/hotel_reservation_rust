use commons::*;

#[derive(Debug, Clone)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub password: String,
    pub favorite: i64,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone)]
pub struct UserService {
    name: String,
    user_db: Collection<Document>,
}

impl UserService {
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
            user_db: {
                let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
                mongo_client_options.app_name = Some(mono_svc::NAME.to_owned());
                let mongo_client = Client::with_options(mongo_client_options)?;
                mongo_client
                    .database(mongo_svc::DB)
                    .collection::<Document>(mongo_svc::coll::USER_PROFILE)
            },
        })
    }
}
