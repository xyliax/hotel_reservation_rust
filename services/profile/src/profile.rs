mod pb;
use commons::mongo_svc::comment::*;
use commons::*;
use pb::profile_api::profile_service_server::*;
use pb::profile_api::*;
use std::collections::HashMap;

static mut GET_COMMENTS_TIMER: Duration = Duration::new(0, 0);

#[derive(Debug, Clone)]
struct ProfileServiceImpl {
    name: String,
    comment_db: Collection<Document>,
    comment_cache: HashMap<String, Vec<Comment>>,
}

impl ProfileServiceImpl {
    /// - Create a new profile service connected to the database.
    async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
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
            comment_cache: HashMap::<String, Vec<Comment>>::new(),
        })
    }
    async fn cache_comments(&mut self, siz: i64) -> Result<(), mongodb::error::Error> {
        let filter = doc! {};
        let mut cursor = match self.comment_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(err) => return Err(err),
        };
        let mut written = 0_i64;
        loop {
            match cursor.try_next().await {
                Ok(opt) => match opt {
                    Some(doc) => match ProfileServiceImpl::doc_to_comment(&doc) {
                        Ok(comment) => {
                            let hotel_id: &String = &comment.hotel_id;
                            match self.comment_cache.get_mut(hotel_id) {
                                Some(comments_for_id) => comments_for_id.push(comment),
                                None => {
                                    self.comment_cache.insert(hotel_id.clone(), vec![comment]);
                                }
                            };
                            written += 1;
                            if written >= siz && siz >= 0_i64 {
                                return Ok(());
                            }
                        }
                        Err(_) => (),
                    },
                    None => break,
                },
                Err(_) => break,
            }
        }
        dbg!(written);
        Ok(())
    }

    /// - Retrieve all comments according to the given hotel id.
    async fn retrieve_comments_by_hotel(&self, hotel_id: &String) -> Option<Vec<Comment>> {
        match self.comment_cache.get(hotel_id) {
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
            match cursor.try_next().await {
                Ok(opt) => match opt {
                    Some(doc) => match ProfileServiceImpl::doc_to_comment(&doc) {
                        Ok(comment) => comment_list.push(comment),
                        Err(_) => (),
                    },
                    None => break,
                },
                Err(_) => break,
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

#[tonic::async_trait]
impl ProfileService for ProfileServiceImpl {
    async fn get_comments(
        &self,
        request: Request<GetCommentsRequest>,
    ) -> Result<Response<GetCommentsResponse>, Status> {
        let start0 = Instant::now();
        let req_inner = request.into_inner();
        let hotel_id = req_inner.hotel_id;
        let comments_for_hotel = match self.retrieve_comments_by_hotel(&hotel_id).await {
            Some(comment_list) => comment_list,
            None => Vec::<Comment>::new(),
        };
        let end0 = Instant::now();
        let get_comments_inner = end0 - start0;
        unsafe {
            GET_COMMENTS_TIMER += get_comments_inner;
            dbg!(GET_COMMENTS_TIMER);
        }
        // println!("{:#?}\n{:#?}", comments_all_hotel.len(), hotel_ids);
        Ok(Response::new(GetCommentsResponse {
            comments: comments_for_hotel,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: std::net::SocketAddr = profile_svc::ADDR.parse()?;
    let mut profile_service_core = ProfileServiceImpl::initialize(profile_svc::NAME).await?;
    profile_service_core.cache_comments(-1_i64).await?;
    println!(
        "{} {} {}",
        profile_service_core.name.red().bold(),
        "listens on".green().bold(),
        format!("{addr}").blue().bold().underline()
    );
    Server::builder()
        .add_service(ProfileServiceServer::new(profile_service_core))
        .serve(addr)
        .await?;
    Ok(())
}
