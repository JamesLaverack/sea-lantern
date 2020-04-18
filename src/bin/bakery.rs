use kube_derive::CustomResource;
use serde::{Deserialize, Serialize};
use log::info;

use kube::{
    api::{Api, ListParams, Meta},
    runtime::Reflector,
    Client,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Vanilla {
    minecraft_version: String,
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug)]
#[kube(group = "minecraft.jameslaverack.com", version = "v1alpha1")]
pub struct MinecraftServerVersionSpec {
    vanilla: Option<Vanilla>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let client = Client::try_default().await?;

    // This example requires `kubectl apply -f examples/foo.yaml` run first

    let versions: Api<MinecraftServerVersion> = Api::all(client);
    let lp = ListParams::default().timeout(20); // low timeout in this example
    let rf = Reflector::new(versions).params(lp);

    let rf2 = rf.clone(); // read from a clone in a task
    tokio::spawn(async move {
        loop {
            // Periodically read our state
            tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
            let crds = rf2
                .state()
                .await
                .unwrap()
                .iter()
                .map(Meta::name)
                .collect::<Vec<_>>();
            info!("Current crds: {:?}", crds);
        }
    });
    rf.run().await?; // run reflector and listen for signals
    Ok(())
}
