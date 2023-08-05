use commons::*;
use geo::pb::geo_api;
use geo_api::geo_service_client::GeoServiceClient;
use geo_api::*;
use monolithic_service::mono_service::MonoService;
use rand::seq::SliceRandom;
use std::env;

async fn retrieve_all_hotel_ids() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let hotel_db = {
        let mut mongo_client_options = ClientOptions::parse(mongo_svc::URL).await?;
        mongo_client_options.app_name = Some(geo_svc::NAME.to_owned());
        let mongo_client = Client::with_options(mongo_client_options)?;
        mongo_client
            .database(mongo_svc::DB)
            .collection::<Document>(mongo_svc::coll::HOTEL)
    };
    let filter = doc! {};
    let mut cursor = hotel_db.find(filter, None).await?;
    let mut hotel_ids = Vec::<String>::new();
    loop {
        if let Ok(Some(doc)) = cursor.try_next().await {
            let hotel_id = doc
                .get_object_id(commons::mongo_svc::hotel::DOC_ID)?
                .to_hex();
            hotel_ids.push(hotel_id);
        } else {
            break;
        }
    }
    Ok(hotel_ids)
}

async fn test_peek_info(item_per_req: i64, req_num: i64) -> Result<(), Box<dyn std::error::Error>> {
    let mut geo_client = GeoServiceClient::connect(geo_svc::PROT).await?;
    let mut mono_client = MonoService::initialize(mono_svc::NAME).await?;
    let hotel_ids = retrieve_all_hotel_ids().await?;
    let mut request_whole = Vec::<Vec<String>>::new();
    for _ in 0..req_num {
        let ids_ref: Vec<&String> = hotel_ids
            .choose_multiple(&mut rand::thread_rng(), item_per_req as usize)
            .collect();
        let mut request_packet = Vec::<String>::new();
        for i in 0..item_per_req {
            request_packet.push(ids_ref[i as usize].clone());
        }
        request_whole.push(request_packet);
    }
    let start0 = Instant::now();
    for packet_num in 0..req_num {
        geo_client
            .peek_info(Request::new(PeekInfoRequest {
                hotel_ids: request_whole[packet_num as usize].clone(),
            }))
            .await?;
    }
    let end0 = Instant::now();
    let peek_info_total_micro = end0 - start0;
    dbg!(peek_info_total_micro);

    let start0 = Instant::now();
    for packet_num in 0..req_num {
        mono_client.geo_service
            .peek_info(&request_whole[packet_num as usize].clone())
            .await?;
    }
    let end0 = Instant::now();
    let peek_info_total_mono = end0 - start0;
    dbg!(peek_info_total_mono);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: test_peek_info <item_per_peek> <peek_num>");
        eprintln!("Example: test_peek_info 10 10");
        return Ok(());
    }
    let item_per_req: i64 = args[1].parse()?;
    let req_num: i64 = args[2].parse()?;
    test_peek_info(item_per_req, req_num).await?;
    Ok(())
}
