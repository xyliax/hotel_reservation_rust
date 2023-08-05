pub mod geo;
pub mod profile;
pub mod rate;
pub mod recommendation;
pub mod research;
pub mod user;
use commons::*;
use geo::HotelInfo;
use geo::*;
use profile::Comment;
use profile::*;
use rate::*;
use recommendation::*;
use research::*;
use user::*;

static mut PEEK_INFO_TIMER: Duration = Duration::new(0, 0);

#[derive(Debug, Clone)]
pub struct MonoService {
    pub name: String,
    pub user_service: UserService,
    pub recomm_service: RecommService,
    pub rate_service: RateService,
    pub profile_service: ProfileService,
    pub resch_service: ReschService,
    pub geo_service: GeoService,
}

impl MonoService {
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
            user_service: UserService::initialize(user_svc::NAME).await?,
            recomm_service: RecommService::initialize(recomm_svc::NAME).await?,
            rate_service: RateService::initialize(rate_svc::NAME).await?,
            profile_service: ProfileService::initialize(profile_svc::NAME).await?,
            resch_service: ReschService::initialize(resch_svc::NAME).await?,
            geo_service: GeoService::initialize(geo_svc::NAME).await?,
        })
    }
}

impl GeoService {
    pub async fn peek_info(&self, hotel_ids: &Vec<String>) -> Result<Vec<HotelInfo>, Status> {
        let start0 = Instant::now();
        let mut locked_cache = self.hotel_info_cache.lock().await;
        let mut hotel_info_list = Vec::<HotelInfo>::new();
        for hotel_id in hotel_ids {
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
        Ok(hotel_info_list)
    }
}
