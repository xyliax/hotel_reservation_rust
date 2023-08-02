mod pb;
use commons::recomm_svc::{POP_THRESHOLD, RECOMM_NUM};
use commons::*;
use pb::recomm_api::recomm_service_server::*;
use pb::recomm_api::*;
use profile::pb::profile_api;
use profile_api::profile_service_client::ProfileServiceClient;
use profile_api::*;
use rate::pb::rate_api;
use rate_api::rate_service_client::*;
use rate_api::*;
use resch_api::resch_service_client::ReschServiceClient;
use resch_api::*;
use research::pb::resch_api;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct RecommServiceImpl {
    name: String,
}

impl RecommServiceImpl {
    /// - Create a new recommendation service.
    async fn initialize(name: &str) -> Result<Self, Error> {
        Ok(Self {
            name: name.to_owned(),
        })
    }
    /// - Process recommendation algortihm
    fn simple_recommend(
        rate_plan_ids: &Vec<String>,
        hotel_distance_list: &Vec<HotelDistance>,
        comments: &Vec<Comment>,
    ) -> Vec<String> {
        // println!("rate_plan: {:#?}", rate_plan_ids);
        // println!("hotel_distance: {:#?}", hotel_distance_list);
        // println!("comments: {:#?}", comments.len());
        let mut recommended_ids = Vec::<String>::new();
        let mut popularity_for_hotel = HashMap::<String, i64>::new();
        for comment in comments {
            let old_value = match popularity_for_hotel.get(&comment.hotel_id) {
                Some(value) => *value,
                None => 0_i64,
            };
            let new_value = old_value + comment.text.len() as i64;
            popularity_for_hotel.insert(comment.hotel_id.clone(), new_value);
        }
        for hotel_id in rate_plan_ids {
            let popularity = match popularity_for_hotel.get(&hotel_id.clone()) {
                Some(value) => *value,
                None => 0_i64,
            };
            if popularity >= POP_THRESHOLD {
                recommended_ids.push(hotel_id.clone());
            }
        }
        let mut distance_map = HashMap::<String, f64>::new();
        for hotel_distance_pair in hotel_distance_list {
            distance_map.insert(
                hotel_distance_pair.hotel_id.clone(),
                hotel_distance_pair.distance,
            );
        }
        recommended_ids
            .sort_by(|hotel_id_1, hotel_id_2| hotel_id_1.partial_cmp(hotel_id_2).unwrap());
        recommended_ids
    }
}

#[tonic::async_trait]
impl RecommService for RecommServiceImpl {
    async fn get_recmd(
        &self,
        request: Request<GetRecmdRequest>,
    ) -> Result<Response<GetRecmdResponse>, Status> {
        let mut resch_client = match ReschServiceClient::connect(resch_svc::PROT).await {
            Ok(client) => client,
            Err(err) => return Err(Status::internal(format!("{:?}", err))),
        };
        let mut rate_client = match RateServiceClient::connect(rate_svc::PROT).await {
            Ok(client) => client,
            Err(err) => return Err(Status::internal(format!("{:?}", err))),
        };
        let mut profile_client = match ProfileServiceClient::connect(profile_svc::PROT).await {
            Ok(client) => client,
            Err(err) => return Err(Status::internal(format!("{:?}", err))),
        };
        let start0 = Instant::now();
        let req_inner = request.into_inner();
        let latitude = req_inner.latitude;
        let longitude = req_inner.longitude;
        let user_profile_opt = req_inner.user_profile;
        let start1 = Instant::now();
        let resch_nearby_reply: ReschNearbyResponse = resch_client
            .resch_nearby(Request::new(ReschNearbyRequest {
                latitude: latitude,
                longitude: longitude,
            }))
            .await?
            .into_inner();
        let end1 = Instant::now();
        let resch_neaby_total = end1 - start1;
        dbg!(resch_neaby_total);
        let start1 = Instant::now();
        let get_rate_plan_reply: GetRatePlanResponse = rate_client
            .get_rate_plan(Request::new(GetRatePlanRequest {
                favorite: match user_profile_opt {
                    Some(user_profile) => user_profile.favorite,
                    None => 0_i64,
                },
                req_num: RECOMM_NUM,
            }))
            .await?
            .into_inner();
        let end1 = Instant::now();
        let get_rate_plan_total = end1 - start1;
        dbg!(get_rate_plan_total);
        let mut nearby_ids = Vec::<String>::new();
        for hotel_distance in resch_nearby_reply.clone().hotel_distance {
            nearby_ids.push(hotel_distance.hotel_id);
        }
        let start1 = Instant::now();
        let get_comments_reply: GetCommentsResponse = profile_client
            .get_comments(Request::new(GetCommentsRequest {
                hotel_ids: nearby_ids,
            }))
            .await?
            .into_inner();
        let end1 = Instant::now();
        let get_comments_total = end1 - start1;
        dbg!(get_comments_total);
        let recommended_ids = RecommServiceImpl::simple_recommend(
            &get_rate_plan_reply.hotel_ids,
            &resch_nearby_reply.hotel_distance,
            &get_comments_reply.comments,
        );
        let end0 = Instant::now();
        let get_recmd_inner = end0 - start0;
        dbg!(get_recmd_inner);
        Ok(Response::new(GetRecmdResponse {
            hotel_ids: recommended_ids,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let addr: std::net::SocketAddr = recomm_svc::ADDR.parse()?;
        let recomm_service_core = RecommServiceImpl::initialize(recomm_svc::NAME).await?;
        /*
        add interceptors here */
        let recomm_service = RecommServiceServer::with_interceptor(
            recomm_service_core.clone(),
            interceptors::_print_request,
        );
        println!(
            "{} {} {}",
            recomm_service_core.name.red().bold(),
            "listens on".green().bold(),
            format!("{addr}").blue().bold().underline()
        );
        match Server::builder()
            .add_service(recomm_service)
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
