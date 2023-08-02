mod pb;
use commons::mongo_svc::comment::*;
use commons::*;
use pb::profile_api::profile_service_server::*;
use pb::profile_api::*;

#[derive(Debug, Clone)]
struct ProfileServiceImpl {
    name: String,
    comment_db: Collection<Document>,
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
        })
    }
    /// - Retrieve all comments according to the given hotel id.
    async fn retrieve_comments_by_hotel(&self, hotel_id: &String) -> Option<Vec<Comment>> {
        let query = doc! {
            HOTEL_ID: hotel_id,
        };
        let mut cursor = match self.comment_db.find(query, None).await {
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
        let req_inner = request.into_inner();
        let hotel_ids = req_inner.hotel_ids;
        let mut comments_all_hotel = Vec::<Comment>::new();
        for hotel_id in &hotel_ids {
            match self.retrieve_comments_by_hotel(&hotel_id).await {
                Some(mut comment_list) => comments_all_hotel.append(&mut comment_list),
                None => (),
            };
            if comments_all_hotel.len() > 10_usize {
                break;
            }
        }
        // println!("{:#?}\n{:#?}", comments_all_hotel.len(), hotel_ids);
        Ok(Response::new(GetCommentsResponse {
            comments: comments_all_hotel,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let addr: std::net::SocketAddr = profile_svc::ADDR.parse()?;
        let profile_service_core = ProfileServiceImpl::initialize(profile_svc::NAME).await?;
        /*
        add interceptors here */
        let profile_service = ProfileServiceServer::with_interceptor(
            profile_service_core.clone(),
            interceptors::_print_request,
        );
        println!(
            "{} {} {}",
            profile_service_core.name.red().bold(),
            "listens on".green().bold(),
            format!("{addr}").blue().bold().underline()
        );
        match Server::builder()
            .add_service(profile_service)
            .serve(addr)
            .await
        {
            Ok(_) => break,
            Err(err) => {
                eprintln!("{}", format!("{err}").red().bold());
                continue;
            }
        }
    }
    Ok(())
}
