pub mod geo;
pub mod profile;
pub mod rate;
pub mod recommendation;
pub mod research;
pub mod user;
use commons::*;
use geo::*;
use profile::*;
use rate::*;
use recomm_svc::*;
use recommendation::*;
use resch_svc::*;
use research::*;
use user::*;

static mut PEEK_INFO_TIMER: Duration = Duration::new(0, 0);
static mut GET_COMMENTS_TIMER: Duration = Duration::new(0, 0);

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

impl UserService {
    pub async fn get_user_profile(&self, username: &String) -> Result<Option<UserProfile>, Status> {
        let start0 = Instant::now();
        let user_profile = match self.retrieve_user_by_name(&username).await {
            Some(user_profile) => user_profile,
            None => return Err(Status::unauthenticated("{username} not Found!")),
        };
        let end0 = Instant::now();
        let get_user_profile_inner = end0 - start0;
        dbg!(get_user_profile_inner);
        Ok(Some(user_profile))
    }
}

impl RecommService {
    pub async fn get_recmd(
        &self,
        latitude: &f64,
        longitude: &f64,
        user_profile_opt: &Option<UserProfile>,
        resch_client: &ReschService,
        rate_client: &RateService,
        profile_client: &ProfileService,
        geo_client: &GeoService,
    ) -> Result<Vec<String>, Status> {
        let start0 = Instant::now();
        let start1 = Instant::now();
        let resch_nearby_reply = resch_client
            .resch_nearby(latitude, longitude, geo_client)
            .await?;
        let end1 = Instant::now();
        let resch_neaby_total = end1 - start1;
        dbg!(resch_neaby_total);
        let start1 = Instant::now();
        let get_rate_plan_reply = rate_client
            .get_rate_plan(
                match user_profile_opt {
                    Some(user_profile) => &user_profile.favorite,
                    None => &0_i64,
                },
                &RECOMM_NUM,
            )
            .await?;
        let end1 = Instant::now();
        let get_rate_plan_total = end1 - start1;
        dbg!(get_rate_plan_total);
        let mut nearby_ids = Vec::<String>::new();
        for hotel_distance in resch_nearby_reply.clone() {
            nearby_ids.push(hotel_distance.hotel_id);
        }
        let mut comments_for_all = Vec::<Comment>::new();
        let start1 = Instant::now();
        for nearby_id in nearby_ids {
            let mut get_comments_reply = profile_client.get_comments(&nearby_id).await?;
            comments_for_all.append(&mut get_comments_reply);
        }
        let end1 = Instant::now();
        let get_comments_total = end1 - start1;
        dbg!(get_comments_total);
        let recommended_ids = RecommService::simple_recommend(
            &get_rate_plan_reply,
            &resch_nearby_reply,
            &comments_for_all,
        );
        let end0 = Instant::now();
        let get_recmd_inner = end0 - start0;
        dbg!(get_recmd_inner);
        Ok(recommended_ids)
    }
}

impl ReschService {
    pub async fn resch_nearby(
        &self,
        latitude: &f64,
        longitude: &f64,
        geo_client: &GeoService,
    ) -> Result<Vec<HotelDistance>, Status> {
        let start0 = Instant::now();
        let start1 = Instant::now();
        let nearby_reply = geo_client.nearby(&NEARBY_NUM, latitude, longitude).await?;
        let end1 = Instant::now();
        let nearby_total = end1 - start1;
        dbg!(nearby_total);
        let result_num = nearby_reply.0;
        let mut hotel_distance = Vec::<HotelDistance>::new();
        for i in 0..usize::try_from(result_num).unwrap() {
            hotel_distance.push(HotelDistance {
                hotel_id: nearby_reply.1[i].clone(),
                distance: nearby_reply.2[i].clone(),
            })
        }
        let end0 = Instant::now();
        let resch_nearby_inner = end0 - start0;
        dbg!(resch_nearby_inner);
        Ok(hotel_distance)
    }
}

impl GeoService {
    pub async fn nearby(
        &self,
        nearby_num: &i64,
        latitude: &f64,
        longitude: &f64,
    ) -> Result<(i64, Vec<String>, Vec<f64>), Status> {
        let start0 = Instant::now();
        let hotel_list: Vec<HotelInfo> = match self.retrieve_hotel_all().await {
            Some(mut hotel_list) => {
                hotel_list.sort_by(|hotel_1, hotel_2| {
                    GeoService::calc_distance(
                        *latitude,
                        *longitude,
                        hotel_1.latitude,
                        hotel_1.longitude,
                    )
                    .partial_cmp(&GeoService::calc_distance(
                        *latitude,
                        *longitude,
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
            if i >= usize::try_from(*nearby_num).unwrap() {
                break;
            }
            hotel_ids_filtered.push(hotel_list[i].id.clone());
            distances_filtered.push(GeoService::calc_distance(
                hotel_list[i].latitude,
                hotel_list[i].longitude,
                *latitude,
                *longitude,
            ));
        }
        let end0 = Instant::now();
        let nearby_inner = end0 - start0;
        dbg!(nearby_inner);
        Ok((
            i64::try_from(hotel_ids_filtered.len()).unwrap(),
            hotel_ids_filtered,
            distances_filtered,
        ))
    }

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

impl ProfileService {
    pub async fn get_comments(&self, hotel_id: &String) -> Result<Vec<Comment>, Status> {
        let start0 = Instant::now();
        let comments_for_hotel = match self.retrieve_comments_by_hotel(hotel_id).await {
            Some(comment_list) => comment_list,
            None => Vec::<Comment>::new(),
        };
        let end0 = Instant::now();
        let get_comments_inner = end0 - start0;
        unsafe {
            GET_COMMENTS_TIMER += get_comments_inner;
            eprintln!(
                "get_comments_inner = {:#?} GET_COMMENTS_TIMER = {:#?}",
                get_comments_inner, GET_COMMENTS_TIMER
            );
        }
        // println!("{:#?}\n{:#?}", comments_all_hotel.len(), hotel_ids);
        Ok(comments_for_hotel)
    }
}

impl RateService {
    pub async fn get_rate_plan(
        &self,
        favorite: &i64,
        req_num: &i64,
    ) -> Result<Vec<String>, Status> {
        let start0 = Instant::now();
        let mut hotel_ids = Vec::<String>::new();
        match self.retrieve_hotel_all().await {
            Some(hotel_list) => {
                for hotel_info in hotel_list {
                    if hotel_ids.len() == *req_num as usize {
                        break;
                    }
                    match RateService::check_satisfy(*favorite, hotel_info.provide) {
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
        Ok(hotel_ids)
    }
}
