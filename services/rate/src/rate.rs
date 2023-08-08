mod pb;
use commons::mongo_svc::hotel::*;
use commons::*;
use pb::rate_api::rate_service_server::*;
use pb::rate_api::*;

#[derive(Debug, Clone)]
struct RateServiceImpl {
    name: String,
    hotel_db: Collection<Document>,
}

impl RateServiceImpl {
    /// - Create a new rate service connected to the database.
    async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
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

    async fn retrieve_hotel_all(&self) -> Option<Vec<HotelInfo>> {
        let filter = doc! {};
        let mut cursor = match self.hotel_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(_) => return None,
        };
        let mut hotel_list = Vec::<HotelInfo>::new();
        loop {
            match cursor.try_next().await {
                Ok(opt) => match opt {
                    Some(doc) => match RateServiceImpl::doc_to_hotel_info(&doc) {
                        Ok(hotel_info) => hotel_list.push(hotel_info),
                        Err(_) => (),
                    },
                    None => break,
                },
                Err(_) => break,
            }
        }
        Some(hotel_list)
    }

    fn doc_to_hotel_info(document: &Document) -> Result<HotelInfo, ValueAccessError> {
        Ok(HotelInfo {
            id: document.get_object_id(DOC_ID)?.to_hex(),
            name: document.get_str(NAME)?.to_owned(),
            latitude: document.get_f64(LATITUDE)?,
            longitude: document.get_f64(LONGITUDE)?,
            provide: document.get_i64(PROVIDE)?,
        })
    }
    /// - Check if a hotel satisfies the user's requirement.
    fn check_satisfy(favorite: i64, provide: i64) -> bool {
        let (res, _) = favorite.overflowing_shr((provide - 1) as u32);
        if favorite == 0 || res & 1 == 1 {
            true
        } else {
            false
        }
    }
}

#[tonic::async_trait]
impl RateService for RateServiceImpl {
    async fn get_rate_plan(
        &self,
        request: Request<GetRatePlanRequest>,
    ) -> Result<Response<GetRatePlanResponse>, Status> {
        let req_inner = request.into_inner();
        let start0 = Instant::now();
        let favorite = req_inner.favorite;
        let req_num = req_inner.req_num;
        let mut hotel_ids = Vec::<String>::new();
        match RateServiceImpl::retrieve_hotel_all(&self).await {
            Some(hotel_list) => {
                for hotel_info in hotel_list {
                    if hotel_ids.len() == req_num as usize {
                        break;
                    }
                    match RateServiceImpl::check_satisfy(favorite, hotel_info.provide) {
                        true => hotel_ids.push(hotel_info.id),
                        false => (),
                    }
                }
            }
            None => (),
        };
        let end0 = Instant::now();
        let get_rate_plan_inner = end0 - start0;
        dbg!(get_rate_plan_inner);
        Ok(Response::new(GetRatePlanResponse {
            hotel_ids: hotel_ids,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: std::net::SocketAddr = rate_svc::ADDR.parse()?;
    let rate_service_core = RateServiceImpl::initialize(rate_svc::NAME).await?;
    println!(
        "{} {} {}",
        rate_service_core.name.red().bold(),
        "listens on".green().bold(),
        format!("{addr}").blue().bold().underline()
    );
    Server::builder()
        .add_service(RateServiceServer::new(rate_service_core))
        .serve(addr)
        .await?;
    Ok(())
}
