mod pb;
use colored::*;
use config::mongo_svc::comment::*;
use config::*;
use futures::stream::TryStreamExt;
use mongodb::bson::document::ValueAccessError;
use mongodb::bson::{doc, Document};
use mongodb::{options::ClientOptions, Client, Collection};
use pb::profile_api::profile_service_server::*;
use pb::profile_api::*;
use tonic::transport::Server;
use tonic::{Request, Response, Result, Status};

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
    async fn retrieve_comments_by_hotel(
        &self,
        hotel_id: &String,
    ) -> Result<Vec<Comment>, mongodb::error::Error> {
        let query = doc! {
            HOTEL_ID: hotel_id,
        };
        let mut cursor = self.comment_db.find(query, None).await?;
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
        Ok(comment_list)
    }

    fn doc_to_comment(document: &Document) -> Result<Comment, ValueAccessError> {
        Ok(Comment {
            id: document.get_object_id(DOC_ID)?.to_hex(),
            hotel_id: document.get_str(HOTEL_ID)?.to_owned(),
            text: document.get_str(TEXT)?.to_owned(),
            date: document.get_str(DATE)?.to_owned(),
            author: document.get_str(AUTHOR)?.to_owned(),
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
        let hotel_id = req_inner.hotel_id;
        let reply = GetCommentsResponse {
            comments: match self.retrieve_comments_by_hotel(&hotel_id).await {
                Ok(comment_list) => comment_list,
                Err(_) => return Err(Status::internal("Error in retrieve_comments_by_hotel")),
            },
        };
        Ok(Response::new(reply))
    }
}

fn _print_request(req: Request<()>) -> Result<Request<()>, Status> {
    println!("intercept: {:#?}", req);
    Ok(req)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let addr: std::net::SocketAddr = profile_svc::ADDR.parse()?;
        let profile_service_core = ProfileServiceImpl::initialize(profile_svc::NAME).await?;
        /*
        add interceptors here */
        let profile_service =
            ProfileServiceServer::with_interceptor(profile_service_core.clone(), _print_request);
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
