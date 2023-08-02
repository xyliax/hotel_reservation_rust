use commons::*;
use recomm_api::recomm_service_client::RecommServiceClient;
use recomm_api::*;
use recommendation::pb::recomm_api;
use tokio;
use user::pb::user_api;
use user_api::user_service_client::UserServiceClient;
use user_api::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut user_client = UserServiceClient::connect(user_svc::PROT).await?;
    let mut recomm_client = RecommServiceClient::connect(recomm_svc::PROT).await?;
    let start0 = Instant::now();
    let user_profile = user_client
        .get_user_profile(Request::new(UserProfileRequest {
            username: "user_02XDuS5q61".to_owned(),
        }))
        .await?
        .into_inner()
        .user_profile
        .unwrap();
    let end0 = Instant::now();
    let get_user_profile_total = end0 - start0;
    dbg!(get_user_profile_total);
    let start0 = Instant::now();
    let recommended_ids = recomm_client
        .get_recmd(Request::new(GetRecmdRequest {
            latitude: user_profile.latitude,
            longitude: user_profile.longitude,
            user_profile: Some(recomm_api::UserProfile {
                id: user_profile.id.clone(),
                username: user_profile.username.clone(),
                password: user_profile.password.clone(),
                favorite: user_profile.favorite,
                latitude: user_profile.latitude,
                longitude: user_profile.longitude,
            }),
        }))
        .await?
        .into_inner()
        .hotel_ids;
    let end0 = Instant::now();
    let get_recmd_total = end0 - start0;
    dbg!(get_recmd_total);
    println!("{:#?}", recommended_ids);
    Ok(())
}
