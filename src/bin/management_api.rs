use tonic::{transport::Server, Request, Response, Status};

use management::minecraft_management_server::{MinecraftManagementServer, MinecraftManagement};
use management::{ListUsersRequest, ListUsersReply, User};

pub mod management {
    tonic::include_proto!("management");
}

#[derive(Debug, Default)]
pub struct DummyMinecraftManagement {}

#[tonic::async_trait]
impl MinecraftManagement for DummyMinecraftManagement {
    async fn list_users(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersReply>, Status> {
        println!("Got a request to list users");

        let reply = management::ListUsersReply {
            online_players: 1,
            max_players: 1337,
            users: [].to_vec(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let minecraft_management = DummyMinecraftManagement::default();

    Server::builder()
        .add_service(MinecraftManagementServer::new(minecraft_management))
        .serve(addr)
        .await?;

    Ok(())
}