use kube_derive::CustomResource;
use serde::{Deserialize, Serialize};
use log::info;

use kube::{
    api::{Api, ListParams, Meta},
    runtime::Reflector,
    runtime::Informer,
    Client,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Vanilla {
    minecraft_version: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Spigot {
    minecraft_version: String,
    spigot_version: String,
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug)]
#[kube(group = "minecraft.jameslaverack.com", version = "v1alpha1")]
pub struct MinecraftServerVersionSpec {
    java_version: String,
    vanilla: Option<Vanilla>,
    spigot: Option<Spigot>
}

struct MinecraftServerVersionController {
    client: Client,
    container_registry: String,
    container_name: String
}

impl MinecraftServerVersionController {
    fn image_name(&self, server_version: &MinecraftServerVersion) -> String {
        return format!("{}/{}:{}", self.container_registry, self.container_name, server_version.name())
    }


    fn reconcile(&self, server_version: &MinecraftServerVersion) {
        info!("Reconciling resource {}", server_version.name())

        // First, figure out the image that we *would* produce for this CRD.
        img = image_name();

        // Ask the registry if we have such an image

        // If yes, update our status as we're done.

        // if not, it should be built.

        // Check to see if we launched a builder pod

        // If no, launch it.

        // If yes, check it's status. If it's a success then set status as pending.
        // If status is failed, fail ourselves.

    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let client = Client::try_default().await?;

    // This example requires `kubectl apply -f examples/foo.yaml` run first

    let versions: Api<MinecraftServerVersion> = Api::all(client.clone());
    let lp = ListParams::default().timeout(20); // low timeout in this example
    let rf = Reflector::new(versions.clone()).params(lp);

    let rf2 = rf.clone(); // read from a clone in a task
    tokio::spawn(async move {
        loop {
            // Periodically read our state. This is pretty terrible, but the use of a reconcile
            // function below abstracts our actual logic from this bad implementation. We can clean
            // this up at a future date and keep the reconcile function the same.
            tokio::time::delay_for(std::time::Duration::from_secs(5)).await;
            rf2.state()
                .await
                .unwrap()
                .iter()
                .for_each(
                    |minecraft_version| {
                        reconcile(minecraft_version, client.clone())
                    }
                );
        }
    });
    rf.run().await?; // run reflector and listen for signals
    Ok(())
}

