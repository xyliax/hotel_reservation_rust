use commons::*;
use mongo_svc::hotel::*;

#[derive(Debug, Clone)]
pub struct HotelInfo {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub provide: i64,
}

#[derive(Debug, Clone)]
pub struct GeoService {
    name: String,
    hotel_db: Collection<Document>,
    pub hotel_info_cache: Cache<HotelInfo>,
}

impl GeoService {
    /// - Create a new geo service connected to the database.
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok({
            let mut geo_service = Self {
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
            };
            let mut geo_service_for_caching = geo_service.clone();
            match geo_service_for_caching.cache_hotel_info(-1_i64).await {
                Ok(_) => (),
                Err(err) => println!("{err}\nUnable to cache hotel info!"),
            }
            geo_service
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
                if let Ok(hotel_info) = GeoService::doc_to_hotel_info(&doc) {
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
                match GeoService::doc_to_hotel_info(&doc) {
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
