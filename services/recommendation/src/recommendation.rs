mod pb;
use config::*;
use pb::recomm_api::recomm_service_server::*;
use pb::recomm_api::*;
use tonic::transport::Server;
use tonic::{Request, Response, Result, Status};

#[derive(Debug)]
struct RecommServiceImpl {
    name: String,
}

#[tonic::async_trait]
impl RecommService for RecommServiceImpl {
    async fn get_recmd(
        &self,
        request: Request<GetRecmdRequest>,
    ) -> Result<Response<GetRecmdResponse>, Status> {
        let req_inner = request.into_inner();
        let latitude = req_inner.latitude;
        let longitude = req_inner.longitude;
        let user_profile = req_inner.user_profile;
        let reply = GetRecmdResponse {
            hotel_ids: Vec::<String>::new(),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let addr = recomm_svc::ADDR.parse()?;
        let recomm_service = RecommServiceImpl {
            name: recomm_svc::NAME.to_owned(),
        };
        println!("{} listens on {}", recomm_service.name, addr);
        match Server::builder()
            .add_service(RecommServiceServer::new(recomm_service))
            .serve(addr)
            .await
        {
            Ok(_) => break,
            Err(err) => {
                eprintln!("{err}");
                continue;
            }
        }
    }
    Ok(())
}
