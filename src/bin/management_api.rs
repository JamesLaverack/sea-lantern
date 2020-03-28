use clap::{App, Arg};

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

    let matches = App::new("Sea Lantern Management API")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Connects to a Minecraft server run using the runtime over a UNIX socket, provides a gRPC API.")
        .arg(Arg::with_name("port")
            .long("grpc-port")
            .required(true)
            .takes_value(true)
            .help("Port to expose gRPC API on"))
        .arg(Arg::with_name("socket")
            .long("socket")
            .required(true)
            .takes_value(true)
            .help("URL of Socket to read/write from"))
        .get_matches();

    let addr = format!("[::1]:{}", matches.value_of("port").unwrap()).parse()?;
    let minecraft_management = DummyMinecraftManagement::default();

    Server::builder()
        .add_service(MinecraftManagementServer::new(minecraft_management))
        .serve(addr)
        .await?;

    Ok(())
}