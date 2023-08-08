mod pb;
use commons::mongo_svc::hotel::*;
use commons::*;
use pb::geo_api::geo_service_server::*;
use pb::geo_api::*;

static mut PEEK_INFO_TIMER: Duration = Duration::new(0, 0);

#[derive(Debug, Clone)]
struct GeoServiceImpl {
    name: String,
    hotel_db: Collection<Document>,
    hotel_info_cache: Cache<HotelInfo>,
}

impl GeoServiceImpl {
    /// - Create a new geo service connected to the database.
    async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
            hotel_db: {
                let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
                mongo_client_options.app_name = Some(geo_svc::NAME.to_owned());
                let mongo_client = Client::with_options(mongo_client_options)?;
                mongo_client
                    .database(mongo_svc::DB)
                    .collection::<Document>(mongo_svc::coll::HOTEL)
            },
            hotel_info_cache: Arc::new(Mutex::new(HashMap::<String, Vec<HotelInfo>>::new())),
        })
    }

    async fn cache_hotel_info(&mut self, siz: i64) -> Result<(), mongodb::error::Error> {
        let mut locked_cache = self.hotel_info_cache.lock().await;
        let filter = doc! {};
        let mut cursor = match self.hotel_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(err) => return Err(err),
        };
        let mut cache_written = 0_i64;
        loop {
            if let Ok(Some(doc)) = cursor.try_next().await {
                if let Ok(hotel_info) = GeoServiceImpl::doc_to_hotel_info(&doc) {
                    let hotel_id: &String = &hotel_info.id;
                    match locked_cache.get_mut(hotel_id) {
                        Some(hotel_info_for_id) => hotel_info_for_id.push(hotel_info),
                        None => {
                            locked_cache.insert(hotel_id.clone(), vec![hotel_info]);
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

    async fn retrieve_hotel_all(&self) -> Option<Vec<HotelInfo>> {
        let filter = doc! {};
        let mut cursor = match self.hotel_db.find(filter, None).await {
            Ok(cursor) => cursor,
            Err(_) => return None,
        };
        let mut hotel_list = Vec::<HotelInfo>::new();
        loop {
            if let Ok(Some(doc)) = cursor.try_next().await {
                match GeoServiceImpl::doc_to_hotel_info(&doc) {
                    Ok(hotel_info) => hotel_list.push(hotel_info),
                    Err(_) => (),
                }
            } else {
                break;
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
    /// - Calculate distance given latitude-longitude coordinates.
    /// - Approximately correct in KM level.
    fn calc_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let earth_radius_meter = 6_378_100_f64;
        let lat1_r = lat1.to_radians();
        let lat2_r = lat2.to_radians();
        let delta_lat = (lat1 - lat2).to_radians();
        let delta_lon = (lon1 - lon2).to_radians();
        let central_angle_inner = (delta_lat / 2.0_f64).sin().powi(2)
            + lat1_r.cos() * lat2_r.cos() * (delta_lon / 2.0_f64).sin().powi(2);
        let central_angle = 2.0_f64 * central_angle_inner.sqrt().asin();
        earth_radius_meter * central_angle
    }
}

#[tonic::async_trait]
impl GeoService for GeoServiceImpl {
    async fn nearby(
        &self,
        request: Request<NearbyRequest>,
    ) -> Result<Response<NearbyResponse>, Status> {
        let req_inner = request.into_inner();
        let start0 = Instant::now();
        let nearby_num = req_inner.nearby_num;
        let latitude = req_inner.latitude;
        let longitude = req_inner.longitude;
        let hotel_list: Vec<HotelInfo> = match self.retrieve_hotel_all().await {
            Some(mut hotel_list) => {
                hotel_list.sort_by(|hotel_1, hotel_2| {
                    GeoServiceImpl::calc_distance(
                        latitude,
                        longitude,
                        hotel_1.latitude,
                        hotel_1.longitude,
                    )
                    .partial_cmp(&GeoServiceImpl::calc_distance(
                        latitude,
                        longitude,
                        hotel_2.latitude,
                        hotel_2.longitude,
                    ))
                    .unwrap()
                });
                hotel_list
            }
            None => Vec::new(),
        };
        let mut hotel_ids_filtered = Vec::<String>::new();
        let mut distances_filtered = Vec::<f64>::new();
        for i in 0..hotel_list.len() {
            if i >= usize::try_from(nearby_num).unwrap() {
                break;
            }
            hotel_ids_filtered.push(hotel_list[i].id.clone());
            distances_filtered.push(GeoServiceImpl::calc_distance(
                hotel_list[i].latitude,
                hotel_list[i].longitude,
                latitude,
                longitude,
            ));
        }
        let end0 = Instant::now();
        let nearby_inner = end0 - start0;
        dbg!(nearby_inner);
        Ok(Response::new(NearbyResponse {
            result_num: i64::try_from(hotel_ids_filtered.len()).unwrap(),
            hotel_ids: hotel_ids_filtered,
            distances: distances_filtered,
        }))
    }

    async fn peek_info(
        &self,
        request: Request<PeekInfoRequest>,
    ) -> Result<Response<PeekInfoResponse>, Status> {
        let req_inner = request.into_inner();
        let hotel_ids = req_inner.hotel_ids;
        let start0 = Instant::now();
        let mut locked_cache = self.hotel_info_cache.lock().await;
        let mut hotel_info_list = Vec::<HotelInfo>::new();
        for hotel_id in &hotel_ids {
            match locked_cache.get_mut(hotel_id) {
                Some(mut hotel_info_for_id) => hotel_info_list.append(&mut hotel_info_for_id),
                None => (),
            }
        }
        let end0 = Instant::now();
        unsafe {
            let peek_info_inner = end0 - start0;
            PEEK_INFO_TIMER += peek_info_inner;
            eprintln!(
                "peek_info_inner = {:#?} PEEK_INFO_TIMER = {:#?}",
                peek_info_inner, PEEK_INFO_TIMER
            );
        }
        Ok(Response::new(PeekInfoResponse {
            hotel_info_list: hotel_info_list,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr: std::net::SocketAddr = geo_svc::ADDR.parse()?;
    let geo_service_core = GeoServiceImpl::initialize(geo_svc::NAME).await?;
    let mut geo_service_for_caching = geo_service_core.clone();
    tokio::spawn(async move {
        match geo_service_for_caching.cache_hotel_info(-1_i64).await {
            Ok(_) => (),
            Err(err) => println!("{err}\nUnable to cache hotel info!"),
        }
    });
    println!(
        "{} {} {}",
        geo_service_core.name.red().bold(),
        "listens on".green().bold(),
        format!("{addr}").blue().bold().underline()
    );
    Server::builder()
        .add_service(GeoServiceServer::new(geo_service_core))
        .serve(addr)
        .await?;
    Ok(())
}
