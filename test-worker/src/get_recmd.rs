use commons::*;
use monolithic_service::mono_service::MonoService;
use recomm_api::recomm_service_client::RecommServiceClient;
use recomm_api::*;
use recommendation::pb::recomm_api;
use std::env;
use user::pb::user_api;
use user_api::user_service_client::UserServiceClient;
use user_api::*;

async fn test_get_recmd(req_num: i64) -> Result<(), Box<dyn std::error::Error>> {
    let mut user_client = UserServiceClient::connect(user_svc::PROT).await?;
    let mut recomm_client = RecommServiceClient::connect(recomm_svc::PROT).await?;
    let mono_client = MonoService::initialize(mono_svc::NAME).await?;
    let user_profile = user_client
        .get_user_profile(Request::new(UserProfileRequest {
            username: "user_02XDuS5q61".to_owned(),
        }))
        .await?
        .into_inner()
        .user_profile
        .unwrap();
    let start0 = Instant::now();
    for _ in 0..req_num {
        let _recommended_ids = recomm_client
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
    }
    let end0 = Instant::now();
    let get_recmd_total_micro = end0 - start0;
    // println!("{:#?}", recommended_ids);
    /*
    Mono-service starts here */
    let start0 = Instant::now();
    for _ in 0..req_num {
        let _recommended_ids = mono_client
            .recomm_service
            .get_recmd(
                &user_profile.latitude,
                &user_profile.longitude,
                &Some(monolithic_service::mono_service::user::UserProfile {
                    id: user_profile.id.clone(),
                    username: user_profile.username.clone(),
                    password: user_profile.password.clone(),
                    favorite: user_profile.favorite,
                    latitude: user_profile.latitude,
                    longitude: user_profile.longitude,
                }),
                &mono_client.resch_service,
                &mono_client.rate_service,
                &mono_client.profile_service,
                &mono_client.geo_service,
            )
            .await?;
    }
    let end0 = Instant::now();
    let get_recmd_total_mono = end0 - start0;
    println!(
        "{req_num}: get_recmd_total_mono = {:#?} get_recmd_total_micro = {:#?}",
        get_recmd_total_mono, get_recmd_total_micro
    );
    // println!("{:#?}", recommended_ids);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: test_get_recmd <req_num>");
        eprintln!("Example: test_get_recmd 100");
        return Ok(());
    }
    let req_num: i64 = args[1].parse()?;
    test_get_recmd(req_num).await?;
    Ok(())
}
