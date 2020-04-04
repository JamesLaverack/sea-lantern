use clap::{App, Arg};

use std::path::{PathBuf};

use log::{error, info, debug};

use tokio::io::AsyncBufReadExt;
use tokio::prelude::*;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::io::BufReader;
use tokio::time::{self, Duration};

use tonic::{transport::Server, Request, Response, Status};

use backup::minecraft_backup_server::{MinecraftBackupServer, MinecraftBackup};
use backup::{BackupRequest, BackupChunk};

use management::minecraft_management_client::MinecraftManagementClient;
use management::{SaveAllRequest, SaveAllReply, DisableAutomaticSaveRequest, DisableAutomaticSaveReply, EnableAutomaticSaveRequest, EnableAutomaticSaveReply};

pub mod management {
    tonic::include_proto!("management");
}

pub mod backup {
    tonic::include_proto!("backup");
}

#[derive(Debug)]
pub struct DummyMinecraftBackup {
    management_api_url :String,
    world_filepath :PathBuf,
}

fn new<P, S>(p :P, u :S) -> DummyMinecraftBackup where P: Into<PathBuf>, S: Into<String>{
    let ui = u.into();
    info!("API URL: {:?}", ui);
    DummyMinecraftBackup{
        world_filepath: p.into(),
        management_api_url: ui,
    }
}

#[tonic::async_trait]
impl MinecraftBackup for DummyMinecraftBackup {

    type BackupStream = mpsc::Receiver<Result<BackupChunk, Status>>;

    async fn backup(
        &self,
        request: Request<BackupRequest>,
    ) -> Result<Response<Self::BackupStream>, Status> {
        info!("Got a backup request");

        let mut management_client = match MinecraftManagementClient::connect(self.management_api_url.clone()).await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to Management API: {:?}", e);
                return Err(Status::unavailable("Unable to connect to Management API"));
            },
        };
        management_client.save_all(tonic::Request::new(SaveAllRequest{})).await?;

        let (mut tx, rx) = mpsc::channel(4);

        tokio::spawn(async move {
            tx.send(Ok(BackupChunk{content: "one".bytes().into_iter().collect()})).await.unwrap();
            tx.send(Ok(BackupChunk{content: "two".bytes().into_iter().collect()})).await.unwrap();
            println!(" /// done sending");
        });

        debug!("Exiting from handler and letting async task do its thing.");
        Ok(Response::new(rx))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();

    let matches = App::new("Sea Lantern Backup API")
        .version("0.1.0")
        .author("James Laverack <james@jameslaverack.com>")
        .about("Connects to a Minecraft server's management API and filesystem, provides a gRPC API.")
        .arg(Arg::with_name("port")
            .long("grpc-port")
            .required(true)
            .takes_value(true)
            .help("Port to expose gRPC API on"))
        .arg(Arg::with_name("management-api-url")
            .long("management-api-url")
            .required(true)
            .takes_value(true)
            .help("URL of management API"))
        .arg(Arg::with_name("world-filepath")
            .long("world-filepath")
            .required(true)
            .takes_value(true)
            .help("Local filesystem path to the Minecraft world"))
        .get_matches();

    let world_filepath = matches.value_of("world-filepath").unwrap();
    info!("Using filepath to world files: {}", world_filepath);
    let management_api_url = matches.value_of("management-api-url").unwrap();
    info!("Using URL to management API: {}", management_api_url);

    let addr = format!("[::1]:{}", matches.value_of("port").unwrap()).parse()?;
    info!("Serving Backup API from: {}", addr);

    Server::builder()
        .add_service(MinecraftBackupServer::new(new(
            world_filepath,
            management_api_url,
        )))
        .serve(addr)
        .await?;

    Ok(())
}