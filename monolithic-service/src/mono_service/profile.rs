use commons::*;
use mongo_svc::comment::*;

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
    pub name: String,
    comment_db: Collection<Document>,
    comment_cache: Cache<Comment>,
}

impl ProfileService {
    /// - Create a new profile service connected to the database.
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok({
            let profile_service = Self {
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
            };
            let mut profile_service_for_caching = profile_service.clone();
            match profile_service_for_caching.cache_comments(-1_i64).await {
                Ok(_) => (),
                Err(err) => println!("{err}\nUnable to cache hotel info!"),
            }
            profile_service
        })
    }

    async fn cache_comments(&mut self, siz: i64) -> Result<(), mongodb::error::Error> {
        let mut locked_cache = self.comment_cache.lock().await;
        let filter = doc! {};
        let mut cursor = match self.comment_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(err) => return Err(err),
        };
        let mut cache_written = 0_i64;
        loop {
            if let Ok(Some(doc)) = cursor.try_next().await {
                if let Ok(comment) = ProfileService::doc_to_comment(&doc) {
                    let hotel_id: &String = &comment.hotel_id;
                    match locked_cache.get_mut(hotel_id) {
                        Some(comments_for_id) => comments_for_id.push(comment),
                        None => {
                            locked_cache.insert(hotel_id.clone(), vec![comment]);
                        }
                    };
                    cache_written += 1;
                    if cache_written >= siz && siz >= 0_i64 {
                        return Ok(());
                    }
                }
            } else {
                break;
            }
        }
        dbg!(cache_written);
        Ok(())
    }

    /// - Retrieve all comments according to the given hotel id.
    pub(in crate::mono_service) async fn retrieve_comments_by_hotel(
        &self,
        hotel_id: &String,
    ) -> Option<Vec<Comment>> {
        let locked_cache = self.comment_cache.lock().await;
        match locked_cache.get(hotel_id) {
            Some(comments_for_id) => {
                return Some(comments_for_id.clone());
            }
            None => {
                eprintln!("{hotel_id}");
            }
        }
        let filter = doc! {
            HOTEL_ID: hotel_id,
        };
        let mut cursor = match self.comment_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(_) => return None,
        };
        let mut comment_list = Vec::<Comment>::new();
        loop {
            if let Ok(Some(doc)) = cursor.try_next().await {
                match ProfileService::doc_to_comment(&doc) {
                    Ok(comment) => comment_list.push(comment),
                    Err(_) => (),
                }
            } else {
                break;
            }
        }
        Some(comment_list)
    }

    fn doc_to_comment(document: &Document) -> Result<Comment, ValueAccessError> {
        Ok(Comment {
            id: document.get_object_id(DOC_ID)?.to_hex(),
            hotel_id: document.get_str(HOTEL_ID)?.to_owned(),
            text: document.get_str(TEXT)?.to_owned(),
            date: document.get_datetime(DATE)?.to_string(),
            author_id: document.get_str(AUTHOR_ID)?.to_owned(),
        })
    }
}
