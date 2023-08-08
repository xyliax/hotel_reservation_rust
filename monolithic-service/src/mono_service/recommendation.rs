use super::profile::Comment;
use super::research::HotelDistance;
use commons::*;
use recomm_svc::*;

#[derive(Debug, Clone)]
pub struct RecommService {
    pub name: String,
}

impl RecommService {
    /// - Create a new recommendation service.
    pub async fn initialize(name: &str) -> Result<Self, mongodb::error::Error> {
        Ok(Self {
            name: name.to_owned(),
        })
    }
    /// - Process recommendation algortihm
    pub(in crate::mono_service) fn simple_recommend(
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
