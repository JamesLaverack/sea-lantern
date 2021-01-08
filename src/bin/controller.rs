use kube_derive::CustomResource;
use serde::{Deserialize, Serialize};
use log::info;
use tokio::stream::StreamExt;

use kube::{
    api::{Api, Meta, WatchEvent},
    runtime::Informer,
    Client,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RconPassword {
    name: String,
    namespace: String,
}

enum Eula {
    Agreed,
    NotAgreed,
}

enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

enum GameMode {
    Survival,
    Creative,
    Spectator,
    Adventure,
}

enum PlayerVsPlayer {
    PvPAllowed,
    PvEOnly,
}

enum Whitelist {
    Disabled,
    Unmanaged,
    Managed,
}

enum LevelType {
    Default,
    Flat,
    LargeBiomes,
    Amplified,
}

struct WorldGeneration {
    seed: Option<String>,
    level_type: LevelType,
}

enum SpawnBehaviour {
    Enabled,
    Disabled,
}

struct Spawning {
    animals: SpawnBehaviour,
    monsters: SpawnBehaviour,
    non_player_characters: SpawnBehaviour,
}

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug)]
#[kube(group = "minecraft.jameslaverack.com", version = "v1alpha1")]
pub struct MinecraftServerSpec {
    rcon_password_secret_ref: Option<RconPassword>,
    motd: String,
    eula: Option<Eula>,
    difficulty: Difficulty,
    game_mode: GameMode,
    view_distance: u8,
    world_size: String,
    maximum_build_height: u32,
    maximum_players: u32,
    pvp: PlayerVsPlayer,
    world_generation: Option<WorldGeneration>,
    whitelist: Whitelist,
    spawning: Spawning,
}

struct MinecraftServerController {
    client: Client,
}

impl MinecraftServerController {

    fn reconcile(&self, server: &MinecraftServer) {
        info!("Reconciling resource {}", server.name());

    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let client = Client::try_default().await?;

    let controller = MinecraftServerController{
        client: client.clone(),
    };

    let server_resources: Api<MinecraftServer> = Api::all(client.clone());
    let inform = Informer::new(server_resources);
    let mut vs = inform.poll().await?;

    while let Some(event) = vs.try_next().await? {
        // Completely ignore the *kind* of event. Just view this as a *change* to this resource.
        let o = match event {
            WatchEvent::Added(K) |
            WatchEvent::Bookmark(K) |
            WatchEvent::Deleted(K) |
            WatchEvent::Modified(K) => {
                K
            },
        };

        let name = Meta::name(&o);
        let namespace = Meta::namespace(&o);

        let api: Api<MinecraftServer> = Api::namespaced(client.clone(), namespace.unwrap().as_str());
        match api.get(name.as_str()) {
          Ok(updated_mc) => {
            controller.reconcile(updated_mc);
          },
          _ => {},
        };
    }

    Ok(())
}

