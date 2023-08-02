mod pb;
use commons::mongo_svc::user::*;
use commons::*;
use pb::user_api::user_service_server::*;
use pb::user_api::*;

#[derive(Debug, Clone)]
struct UserServiceImpl {
    name: String,
    user_db: Collection<Document>,
}

impl UserServiceImpl {
    /// - Create a new user service connected to the database.
    async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
            user_db: {
                let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
                mongo_client_options.app_name = Some(user_svc::NAME.to_owned());
                let mongo_client = Client::with_options(mongo_client_options)?;
                mongo_client
                    .database(mongo_svc::DB)
                    .collection::<Document>(mongo_svc::coll::USER_PROFILE)
            },
        })
    }
    /// - Retrieve an optional user profile according to the given name.
    async fn retrieve_user_by_name(&self, name: &String) -> Option<UserProfile> {
        let filter = doc! {
            "username": name
        };
        let res = self.user_db.find_one(filter, None).await;
        match res {
            Ok(option) => match option {
                Some(doc) => match doc {
                    _ => match UserServiceImpl::doc_to_user_profile(&doc) {
                        Ok(user_profile) => Some(user_profile),
                        Err(_) => None,
                    },
                },
                None => None,
            },
            Err(_) => None,
        }
    }
    /**
     * Retrieve all user profiles.
     * TODO: filter
     */
    async fn _retrieve_user_all(&self) -> Option<Vec<Option<UserProfile>>> {
        let filter = doc! {};
        let mut cursor = match self.user_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(_) => return None,
        };
        let mut user_list = Vec::<Option<UserProfile>>::new();
        loop {
            match cursor.try_next().await {
                Ok(opt) => match opt {
                    Some(doc) => match UserServiceImpl::doc_to_user_profile(&doc) {
                        Ok(user_profile) => user_list.push(Some(user_profile)),
                        Err(_) => user_list.push(None),
                    },
                    None => break,
                },
                Err(_) => break,
            }
        }
        Some(user_list)
    }

    fn _user_profile_to_doc(user_profile: &UserProfile) -> Document {
        doc! {
            USERNAME: &user_profile.username,
            PASSWORD: &user_profile.password,
            FAVORITE: &user_profile.favorite,
            LATITUDE: &user_profile.latitude,
            LONGITUDE: &user_profile.longitude,
        }
    }

    fn doc_to_user_profile(document: &Document) -> Result<UserProfile, ValueAccessError> {
        Ok(UserProfile {
            id: document.get_object_id(DOC_ID)?.to_hex(),
            username: document.get_str(USERNAME)?.to_owned(),
            password: document.get_str(PASSWORD)?.to_owned(),
            favorite: document.get_i64(FAVORITE)?,
            latitude: document.get_f64(LATITUDE)?,
            longitude: document.get_f64(LONGITUDE)?,
        })
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn check_user(
        &self,
        request: Request<CheckUserRequest>,
    ) -> Result<Response<CheckUserResponse>, Status> {
        let req_inner = request.into_inner();
        let username = req_inner.username;
        let password = req_inner.password;
        match self.retrieve_user_by_name(&username).await {
            Some(user_profile) => {
                if user_profile.password == password {
                    Ok(Response::new(CheckUserResponse { correct: true }))
                } else {
                    Err(Status::unauthenticated("Incorrect Password!"))
                }
            }
            None => Err(Status::unauthenticated("{username} not Found!")),
        }
    }

    async fn get_user_profile(
        &self,
        request: Request<UserProfileRequest>,
    ) -> Result<Response<UserProfileResponse>, Status> {
        // let parent_cx = global::get_text_map_propagator(|prop| {
        //     prop.extract(&MetadataMap(request.metadata_mut()))
        // });
        // let mut span =
        //     global::tracer("get_user_profile").start_with_context("Processing Reply", parent_cx);
        let start0 = Instant::now();
        let req_inner = request.into_inner();
        let username = req_inner.username;
        let user_profile = match self.retrieve_user_by_name(&username).await {
            Some(user_profile) => user_profile,
            None => return Err(Status::unauthenticated("{username} not Found!")),
        };
        let end0 = Instant::now();
        let get_user_profile_inner = end0 - start0;
        dbg!(get_user_profile_inner);
        Ok(Response::new(UserProfileResponse {
            user_profile: Some(user_profile),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let _tracer = tracing::tracing_init(&user_svc::NAME.to_owned());
        let addr: std::net::SocketAddr = user_svc::ADDR.parse()?;
        let user_service_core = UserServiceImpl::initialize(user_svc::NAME).await?;
        /*
        add interceptors here */
        let user_service = UserServiceServer::with_interceptor(
            user_service_core.clone(),
            interceptors::_print_request,
        );
        println!(
            "{} {} {}",
            user_service_core.name.red().bold(),
            "listens on".green().bold(),
            format!("{addr}").blue().bold().underline()
        );
        match Server::builder()
            .add_service(user_service)
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
