mod pb;
use commons::resch_svc::NEARBY_NUM;
use commons::*;
use geo::pb::geo_api;
use geo_api::geo_service_client::GeoServiceClient;
use geo_api::*;
use pb::resch_api::resch_service_server::*;
use pb::resch_api::*;

#[derive(Debug, Clone)]
struct ReschServiceImpl {
    name: String,
}

impl ReschServiceImpl {
    /// - Create a new research service connected to the database.
    async fn initialize(name: &str) -> Result<Self, Error> {
        Ok(Self {
            name: name.to_owned(),
        })
    }
}

#[tonic::async_trait]
impl ReschService for ReschServiceImpl {
    async fn resch_nearby(
        &self,
        request: Request<ReschNearbyRequest>,
    ) -> Result<Response<ReschNearbyResponse>, Status> {
        let start0 = Instant::now();
        let mut geo_client = match GeoServiceClient::connect(geo_svc::PROT).await {
            Ok(client) => client,
            Err(err) => return Err(Status::internal(format!("{:?}", err))),
        };
        let req_inner = request.into_inner();
        let latitude = req_inner.latitude;
        let longitude = req_inner.longitude;
        let start1 = Instant::now();
        let nearby_reply: NearbyResponse = geo_client
            .nearby(Request::new(NearbyRequest {
                nearby_num: NEARBY_NUM,
                latitude: latitude,
                longitude: longitude,
            }))
            .await?
            .into_inner();
        let end1 = Instant::now();
        let nearby_total = end1 - start1;
        dbg!(nearby_total);
        let result_num = nearby_reply.result_num;
        let mut hotel_distance = Vec::<HotelDistance>::new();
        for i in 0..usize::try_from(result_num).unwrap() {
            hotel_distance.push(HotelDistance {
                hotel_id: nearby_reply.hotel_ids[i].clone(),
                distance: nearby_reply.distances[i].clone(),
            })
        }
        let end0 = Instant::now();
        let resch_nearby_inner = end0 - start0;
        dbg!(resch_nearby_inner);
        Ok(Response::new(ReschNearbyResponse {
            hotel_distance: hotel_distance,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let addr: std::net::SocketAddr = resch_svc::ADDR.parse()?;
        let resch_service_core = ReschServiceImpl::initialize(resch_svc::NAME).await?;
        /*
        add interceptors here */
        let resch_service = ReschServiceServer::with_interceptor(
            resch_service_core.clone(),
            interceptors::_print_request,
        );
        println!(
            "{} {} {}",
            resch_service_core.name.red().bold(),
            "listens on".green().bold(),
            format!("{addr}").blue().bold().underline()
        );
        match Server::builder()
            .add_service(resch_service)
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
