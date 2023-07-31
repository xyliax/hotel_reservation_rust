mod pb;
use config::*;
use pb::reserv_api::reserv_service_server::*;
use pb::reserv_api::*;
use tonic::transport::{Channel, Server};
use tonic::{Request, Response, Result, Status};
use user::pb::user_api;
use user_api::user_service_client::UserServiceClient;
use user_api::CheckUserRequest;

#[derive(Debug, Clone)]
struct ReservServiceImpl {
    name: String,
    user_client: UserServiceClient<Channel>,
}

#[tonic::async_trait]
impl ReservService for ReservServiceImpl {
    async fn make_reservation(
        &self,
        _request: Request<ReservRequest>,
    ) -> Result<Response<ReservResponse>, Status> {
        let mut htls = Vec::new();
        htls.push("hotel_make_reservation".to_owned());
        let reply = ReservResponse { hotel_id: htls };
        Ok(Response::new(reply))
    }
    async fn check_availability(
        &self,
        _request: Request<ReservRequest>,
    ) -> Result<Response<ReservResponse>, Status> {
        let mut htls = Vec::new();
        htls.push("hotel_check_availability".to_owned());
        let reply = ReservResponse { hotel_id: htls };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let addr = reserv_svc::ADDR.parse()?;
        let mut reserv_service = ReservServiceImpl {
            name: reserv_svc::NAME.to_owned(),
            user_client: UserServiceClient::connect(user_svc::PROT).await?,
        };
        reserv_service
            .user_client
            .check_user(Request::new(CheckUserRequest {
                username: "".to_owned(),
                password: "".to_owned(),
            }))
            .await?;
        println!("{} listens on {}", reserv_service.name, addr);
        match Server::builder()
            .add_service(ReservServiceServer::new(reserv_service))
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
