pub mod error;

mod containers;
mod prelude;
mod stderr;
mod stdout;

use crate::containers::ContainerRuntime;
use crate::error::Error;

use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use stderr::StdErr;
use stdout::StdOut;
use tokio::sync::Mutex;
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Result, Store};
use wasmtime_wasi::{async_trait, DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiView};

bindgen!({
    world: "extension",
    async: true,
});

#[derive(Clone)]
pub struct Station {
    container_runtime: Arc<Mutex<ContainerRuntime>>,
    engine: Engine,
    linker: Linker<State>,
}

pub struct RunArgs<'a> {
    pub root_dir: PathBuf,
    pub wasm: &'a [u8],
    pub name: &'a str,
    pub node_name: &'a str,
    pub gateway: &'a str,
    pub mqtt_port: u16,
}

struct State {
    ctx: WasiCtx,
    table: ResourceTable,
    container_runtime: Arc<Mutex<ContainerRuntime>>,
    root_dir: PathBuf,
}

impl Station {
    pub fn new() -> Result<Self, Error> {
        let engine = {
            let mut config = Config::new();
            config.async_support(true);
            Engine::new(&config)?
        };
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        Extension::add_to_linker(&mut linker, |s| s)?;
        Ok(Self {
            engine,
            linker,
            container_runtime: Arc::new(Mutex::new(ContainerRuntime::new()?)),
        })
    }

    pub async fn run<'a>(
        &self,
        args: RunArgs<'a>,
    ) -> Result<(), Error> {
        let component = Component::new(&self.engine, args.wasm)?;
        let uname = nix::sys::utsname::uname()?;
        std::fs::create_dir_all(&args.root_dir)?;
        let ctx = WasiCtxBuilder::new()
            .stdout(StdOut::new())
            .stderr(StdErr::new())
            .env("NAME", args.name)
            .env("NODE_NAME", uname.nodename().to_string_lossy())
            .env("GATEWAY", args.gateway)
            .env("MQTT_PORT", args.mqtt_port.to_string())
            .env("KERNEL", uname.release().to_string_lossy())
            .allow_udp(true)
            .allow_tcp(true)
            .socket_addr_check(Box::new(|address, address_use| {
                Box::pin(async move {
                    tracing::info!("WASM connecting to {address}/{address_use:?}");
                    true
                }) as Pin<Box<dyn Future<Output = bool> + Send + Sync>>
            }))
            .preopened_dir(&args.root_dir, "/", DirPerms::all(), FilePerms::all())?
            .build();
        let mut store = Store::new(
            &self.engine,
            State {
                ctx,
                table: ResourceTable::new(),
                container_runtime: self.container_runtime.clone(),
                root_dir: args.root_dir.into(),
            },
        );
        let bindings = Extension::instantiate_async(&mut store, &component, &self.linker).await?;
        bindings
            .call_run(&mut store)
            .await?
            .map_err(Error::Runtime)?;
        Ok(())
    }
}

#[async_trait]
impl ExtensionImports for State {
    async fn download_image(
        &mut self,
        path: String,
        uri: String,
    ) -> Result<(), String> {
        let client = lararium_amphora::Client::new(
            "https://index.docker.io/",
            Path::new("/tmp/lararium/cache"),
        );
        client
            .download(&PathBuf::from(path), &uri)
            .await
            .map_err(|_| String::from("Failed to download image"))?;
        Ok(())
    }

    async fn create_container(
        &mut self,
        args: CreateContainerArgs,
    ) -> Result<(), String> {
        let root_dir = PathBuf::from(args.root_dir);
        let root_dir = root_dir
            .strip_prefix("/")
            .map_err(|_| String::from("root dir must be absolute"))?;
        let root_dir = self.root_dir.join(root_dir);
        self.container_runtime.lock().await.add(
            args.name,
            containers::ContainerBlueprint {
                root_dir,
                work_dir: args.work_dir.into(),
                command: args.command,
                args: args.args,
                env: args.env,
                wayland: args.wayland,
            },
        );
        Ok(())
    }

    async fn run_container(
        &mut self,
        name: String,
    ) -> Result<(), String> {
        self.container_runtime.lock().await.run(&name).await;
        Ok(())
    }

    async fn kill_container(
        &mut self,
        name: String,
    ) -> Result<(), String> {
        self.container_runtime.lock().await.kill(&name).await;
        Ok(())
    }

    async fn mount_local_volume(
        &mut self,
        path: String,
        name: String,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn mount_shared_volume(
        &mut self,
        path: String,
        name: String,
    ) -> Result<(), String> {
        let path = PathBuf::from(path);
        let path = path
            .strip_prefix("/")
            .map_err(|_| String::from("path must be absolute"))?;
        let path = self.root_dir.join(path);
        fs::create_dir_all(path).map_err(|_| String::from("failed to create directory"))?;
        Ok(())
    }

    async fn mount_temporary_volume(
        &mut self,
        path: String,
    ) -> Result<(), String> {
        Ok(())
    }
}

impl WasiView for State {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}
