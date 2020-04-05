use clap::{App, Arg};

use std::path::{PathBuf};

use log::{error, info, debug};

use flate2::Compression;
use flate2::write::GzEncoder;

use tokio::fs::File;
use tokio::io::AsyncReadExt;

use std::convert::Infallible;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};

use tokio::io::AsyncBufReadExt;
use tokio::prelude::*;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::io::BufReader;
use tokio::time::{self, Duration};

use tonic::{Request as TonicRequest, Response as TonicResponse};

use management::minecraft_management_client::MinecraftManagementClient;
use management::{SaveAllRequest, SaveAllReply, DisableAutomaticSaveRequest, DisableAutomaticSaveReply, EnableAutomaticSaveRequest, EnableAutomaticSaveReply};

pub mod management {
    tonic::include_proto!("management");
}

static INDEX: &str = "examples/send_file_index.html";
static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";

#[derive(Debug)]
pub struct BackupSvc {
    management_api_url :String,
    world_filepath :PathBuf,
}

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type HyperResult<T> = std::result::Result<T, GenericError>;

impl BackupSvc {
    fn new<P, S>(p :P, u :S) -> BackupSvc where P: Into<PathBuf>, S: Into<String>{
        BackupSvc {
            world_filepath: p.into(),
            management_api_url: u.into(),
        }
    }

    async fn backup(&self, _req: Request<Body>) -> HyperResult<Response<Body>> {
        info!("Got a backup request");

        let mut management_client = match MinecraftManagementClient::connect(self.management_api_url.clone()).await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to Management API: {:?}", e);
                let response = Response::builder()
                    .status(StatusCode::InternalServerError)
                    .body("Unable to connect to Management API")?;
                Ok(response);
            },
        };
        /*
        tokio::spawn(async move {
            management_client.save_all(tonic::Request::new(SaveAllRequest {})).await?;
            management_client.disable_automatic_save(tonic::Request::new(DisableAutomaticSaveRequest {})).await?;

            let enc = GzEncoder::new(tx, Compression::default());
            let mut tar = tar::Builder::new(enc);
            tar.append_dir_all("", self.world_filepath.clone())?;

            println!(" /// done sending");

            management_client.enable_automatic_save(tonic::Request::new(EnableAutomaticSaveRequest {})).await
        });
        */
        debug!("Exiting from handler and letting async task do its thing.");
        Ok(Response::new("test".into()))
    }
}


async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
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

    let addr :std::net::SocketAddr = format!("[::1]:{}", matches.value_of("port").unwrap()).parse()?;
    info!("Serving Backup API from: {:?}", addr);

    let backup_svc = BackupSvc::new(
        world_filepath,
        management_api_url,
    );

    /*
    let make_service =
        make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(backup_svc.clone().backup())) });

    let server = Server::bind(&addr).serve(make_service);
    */

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(|r| async { backup_svc.backup(r) }))
    });

    let server = Server::bind(&addr)
        .serve(make_svc);


    println!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}
