mod pb;
use commons::mongo_svc::hotel::*;
use commons::*;
use pb::geo_api::geo_service_server::*;
use pb::geo_api::*;

#[derive(Debug, Clone)]
struct GeoServiceImpl {
    name: String,
    hotel_db: Collection<Document>,
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
                    Some(doc) => match GeoServiceImpl::doc_to_hotel_info(&doc) {
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
    /// - Calculate distance given latitude-longitude coordinates.
    /// - Approximately correct in KM level.
    fn calc_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let earth_radius_meter = 6_378_100_f64;
        let lat1_r = lat1.to_radians();
        let lat2_r = lat2.to_radians();
        let delta_lat = (lat1 - lat2).to_radians();
        let delta_lon = (lon1 - lon2).to_radians();
        let central_angle_inner = (delta_lat / 2.0).sin().powi(2)
            + lat1_r.cos() * lat2_r.cos() * (delta_lon / 2.0).sin().powi(2);
        let central_angle = 2.0 * central_angle_inner.sqrt().asin();
        earth_radius_meter * central_angle
    }
}

#[tonic::async_trait]
impl GeoService for GeoServiceImpl {
    async fn nearby(
        &self,
        request: Request<NearbyRequest>,
    ) -> Result<Response<NearbyResponse>, Status> {
        let start0 = Instant::now();
        let req_inner = request.into_inner();
        let nearby_num = req_inner.nearby_num;
        let latitude = req_inner.latitude;
        let longitude = req_inner.longitude;
        let hotel_list: Vec<HotelInfo> = match GeoServiceImpl::retrieve_hotel_all(&self).await {
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
        _request: Request<PeekInfoRequest>,
    ) -> Result<Response<PeekInfoResponse>, Status> {
        Ok(Response::new(PeekInfoResponse {
            hotel_info_list: Vec::new(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let addr: std::net::SocketAddr = geo_svc::ADDR.parse()?;
        let geo_service_core = GeoServiceImpl::initialize(geo_svc::NAME).await?;
        /*
        add interceptors here */
        let geo_service = GeoServiceServer::with_interceptor(
            geo_service_core.clone(),
            interceptors::_print_request,
        );
        println!(
            "{} {} {}",
            geo_service_core.name.red().bold(),
            "listens on".green().bold(),
            format!("{addr}").blue().bold().underline()
        );
        match Server::builder().add_service(geo_service).serve(addr).await {
            Ok(_) => break,
            Err(err) => {
                eprintln!("{}", format!("{err}").red().bold());
                continue;
            }
        }
    }
    Ok(())
}
