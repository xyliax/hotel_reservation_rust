use commons::*;

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: String,
    pub hotel_id: String,
    pub text: String,
    pub date: String,
    pub author_id: String,
}

#[derive(Debug, Clone)]
pub struct ProfileService {
    name: String,
    comment_db: Collection<Document>,
    comment_cache: Cache<Comment>,
}

impl ProfileService {
    /// - Create a new profile service connected to the database.
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
            comment_db: {
                let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
                mongo_client_options.app_name = Some(profile_svc::NAME.to_owned());
                let mongo_client = Client::with_options(mongo_client_options)?;
                mongo_client
                    .database(mongo_svc::DB)
                    .collection::<Document>(mongo_svc::coll::COMMENT)
            },
            comment_cache: Arc::new(Mutex::new(HashMap::<String, Vec<Comment>>::new())),
        })
    }
}
