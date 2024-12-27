mod error;

pub use error::Error;

use crate::prelude::*;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::mount::{mount, umount, MsFlags};
use nix::sched::{self, CloneFlags};
use nix::sys::signal::{kill, Signal};
use nix::sys::stat::{fchmod, Mode};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, dup2, fchown, pipe, ForkResult, Gid, Uid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::os::fd::IntoRawFd;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

#[derive(Clone)]
pub struct ContainerRuntime {
    blueprints: HashMap<String, ContainerBlueprint>,
    handles: Arc<RwLock<HashMap<String, ContainerHandle>>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct ContainerBlueprint {
    pub root_dir: PathBuf,
    pub work_dir: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub wayland: bool,
}

pub struct ContainerHandle {
    signal_tx: Option<oneshot::Sender<Signal>>,
}

struct ContainerInstance {
    root_dir: PathBuf,
    work_dir: PathBuf,
    command: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    hostname: String,
    username: String,
    gid: Gid,
    uid: Uid,
    wayland: bool,
    signal_rx: oneshot::Receiver<Signal>,
}

impl ContainerRuntime {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            blueprints: HashMap::new(),
            handles: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn add(
        &mut self,
        name: impl Into<String>,
        blueprint: ContainerBlueprint,
    ) {
        let _ = self.blueprints.insert(name.into(), blueprint);
    }

    pub async fn run(
        &self,
        name: &str,
    ) {
        tracing::debug!("Starting container {name}");
        let blueprint = self.blueprints.get(name).unwrap();
        let handle = blueprint.run(name).unwrap();
        self.handles.write().await.insert(name.to_string(), handle);
    }

    pub async fn kill(
        &self,
        name: &str,
    ) {
        tracing::debug!("Killing container {name}");
        self.handles.write().await.remove(name);
    }
}

impl ContainerBlueprint {
    pub fn run(
        &self,
        hostname: impl Into<String>,
    ) -> Result<ContainerHandle, Error> {
        let (signal_tx, signal_rx) = oneshot::channel();

        let container = ContainerInstance {
            root_dir: self.root_dir.clone(),
            work_dir: self.work_dir.clone(),
            hostname: hostname.into(),
            username: "lararium".into(),
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            gid: Gid::from_raw(1000),
            uid: Uid::from_raw(1000),
            wayland: self.wayland,
            signal_rx,
        };

        tokio::task::spawn_blocking(move || container.run().unwrap());

        Ok(ContainerHandle {
            signal_tx: Some(signal_tx),
        })
    }
}

impl Drop for ContainerHandle {
    fn drop(&mut self) {
        if let Some(signal_tx) = self.signal_tx.take() {
            let _ = signal_tx.send(Signal::SIGTERM);
        }
    }
}

impl ContainerInstance {
    fn run(mut self) -> Result<(), Error> {
        let (stdout_read, stdout_write) = pipe().unwrap();
        let (stderr_read, stderr_write) = pipe().unwrap();

        fcntl(
            stdout_read.as_raw_fd(),
            FcntlArg::F_SETFL(OFlag::O_NONBLOCK),
        )
        .unwrap();
        fcntl(
            stderr_read.as_raw_fd(),
            FcntlArg::F_SETFL(OFlag::O_NONBLOCK),
        )
        .unwrap();

        let cgroup_path = Path::new("/sys/fs/cgroup/lararium").join(&self.hostname);
        fs::create_dir_all(&cgroup_path).unwrap();

        fs::create_dir_all(self.root_dir.join(&self.work_dir)).unwrap();
        fs::create_dir_all(self.root_dir.join("proc")).unwrap();
        fs::create_dir_all(self.root_dir.join("root")).unwrap();
        fs::create_dir_all(self.root_dir.join("sys")).unwrap();
        fs::create_dir_all(self.root_dir.join("tmp")).unwrap();
        fs::create_dir_all(self.root_dir.join("dev/dri")).unwrap();
        fs::create_dir_all(self.root_dir.join("dev/input")).unwrap();
        fs::create_dir_all(self.root_dir.join("dev/snd")).unwrap();
        fs::create_dir_all(self.root_dir.join("etc")).unwrap();
        fs::File::create(self.root_dir.join("dev/null")).unwrap();
        fs::File::create(self.root_dir.join("dev/random")).unwrap();
        fs::File::create(self.root_dir.join("dev/urandom")).unwrap();

        let run_user_dir = self.root_dir.join("run/user").join(self.uid.to_string());
        fs::create_dir_all(&run_user_dir).unwrap();
        {
            let file = fs::File::open(&run_user_dir).unwrap();
            fchown(file.as_raw_fd(), Some(self.uid), Some(self.gid)).unwrap();
            fchmod(file.as_raw_fd(), Mode::from_bits(0o700).unwrap()).unwrap();
        }

        let wayland_socket = run_user_dir.join("wayland-1");
        fs::File::create(&wayland_socket).unwrap();
        {
            let file = fs::File::open(wayland_socket).unwrap();
            fchown(file.as_raw_fd(), Some(self.uid), Some(self.gid)).unwrap();
            fchmod(file.as_raw_fd(), Mode::from_bits(0o700).unwrap()).unwrap();
        }

        let pipewire_socket = run_user_dir.join("pipewire-0");
        fs::File::create(&pipewire_socket).unwrap();
        {
            let file = fs::File::open(pipewire_socket).unwrap();
            fchown(file.as_raw_fd(), Some(self.uid), Some(self.gid)).unwrap();
            fchmod(file.as_raw_fd(), Mode::from_bits(0o700).unwrap()).unwrap();
        }

        let home_dir = self.root_dir.join("home").join(&self.username);
        fs::create_dir_all(&home_dir).unwrap();
        {
            let file = fs::File::open(home_dir).unwrap();
            fchown(file.as_raw_fd(), Some(self.uid), Some(self.gid)).unwrap();
            fchmod(file.as_raw_fd(), Mode::from_bits(0o751).unwrap()).unwrap();
        }

        fs::write(
            self.root_dir.join("etc/hostname"),
            format!("{}\n", self.hostname),
        )
        .unwrap();
        fs::write(
            self.root_dir.join("etc/group"),
            format!(
                "root:x:0:\n{username}:x:{gid}:\n",
                username = self.username,
                gid = self.gid,
            ),
        )
        .unwrap();
        fs::write(
            self.root_dir.join("etc/passwd"),
            format!("root:x:0:0:root:/root:/bin/sh\n{username}:x:{uid}:{gid}:{username}:/home/{username}:/bin/sh\n", username = self.username, uid = self.uid, gid = self.gid),
        ).unwrap();
        fs::write(self.root_dir.join("etc/resolv.conf"), "nameserver 1.1.1.1").unwrap();

        sched::unshare(
            CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWUTS,
        )
        .unwrap();

        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child }) => {
                drop(stdout_write);
                drop(stderr_write);

                loop {
                    let mut buffer = [0; 1024];

                    match nix::unistd::read(stdout_read.as_raw_fd(), &mut buffer) {
                        Ok(bytes_read) if bytes_read > 0 => {
                            let stdout = String::from_utf8_lossy(&buffer[..bytes_read]);
                            print!("{stdout}");
                        }
                        Ok(_) => {}
                        Err(nix::errno::Errno::EAGAIN) => {}
                        Err(err) => panic!("Error reading stdout: {:?}", err),
                    }

                    match nix::unistd::read(stderr_read.as_raw_fd(), &mut buffer) {
                        Ok(bytes_read) if bytes_read > 0 => {
                            let stderr = String::from_utf8_lossy(&buffer[..bytes_read]);
                            eprint!("{stderr}");
                        }
                        Ok(_) => {}
                        Err(nix::errno::Errno::EAGAIN) => {}
                        Err(err) => panic!("Error reading stderr: {:?}", err),
                    }

                    if let Ok(signal) = self.signal_rx.try_recv() {
                        if let Err(error) = kill(child, signal) {
                            error!("Failed to kill child process: {error}");
                        } else {
                            info!("Killed container with signal {signal}");
                        }
                    }

                    match waitpid(child, Some(nix::sys::wait::WaitPidFlag::WNOHANG)).unwrap() {
                        WaitStatus::StillAlive | WaitStatus::Continued(_) => {}
                        WaitStatus::Exited(_, exit_code) => {
                            if exit_code != 0 {
                                error!("Container exited with code {exit_code}");
                            } else {
                                info!("Container exited successfully.");
                            }
                            break;
                        }
                        WaitStatus::Signaled(_, _, _) => {
                            info!("Container killed by signal.");
                            break;
                        }
                        _ => {
                            error!("Container exited with unknown status");
                            break;
                        }
                    };
                }

                if let Err(error) = fs::remove_dir(&cgroup_path) {
                    error!("Failed to remove cgroup: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("proc")) {
                    error!("Failed to unmount /proc: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("sys")) {
                    error!("Failed to unmount /sys: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("tmp")) {
                    error!("Failed to unmount /tmp: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("dev/dri")) {
                    error!("Failed to unmount /dev/dri: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("dev/input")) {
                    error!("Failed to unmount /dev/input: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("dev/null")) {
                    error!("Failed to unmount /dev/null: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("dev/random")) {
                    error!("Failed to unmount /dev/random: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("dev/snd")) {
                    error!("Failed to unmount /dev/snd: {error}");
                }
                if let Err(error) = umount(&self.root_dir.join("dev/urandom")) {
                    error!("Failed to unmount /dev/urandom: {error}");
                }
                if self.wayland {
                    if let Err(error) = umount(&run_user_dir.join("wayland-1")) {
                        error!("Failed to unmount wayland socket: {error}");
                    }
                }
                if let Err(error) = umount(&run_user_dir.join("pipewire-0")) {
                    error!("Failed to unmount pipewire socket: {error}");
                }
                Ok(())
            }
            Ok(ForkResult::Child) => {
                drop(stdout_read);
                drop(stderr_read);
                dup2(stdout_write.into_raw_fd(), 1).unwrap();
                dup2(stderr_write.into_raw_fd(), 2).unwrap();

                mount(
                    Some("proc"),
                    &self.root_dir.join("proc"),
                    Some("proc"),
                    MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
                    None::<&str>,
                )
                .unwrap();

                mount(
                    Some("sysfs"),
                    &self.root_dir.join("sys"),
                    Some("sysfs"),
                    MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
                    None::<&str>,
                )
                .unwrap();

                mount(
                    Some("tmpfs"),
                    &self.root_dir.join("tmp"),
                    Some("tmpfs"),
                    MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
                    None::<&str>,
                )
                .unwrap();

                mount(
                    Some("/dev/dri"),
                    &self.root_dir.join("dev/dri"),
                    None::<&str>,
                    MsFlags::MS_BIND | MsFlags::MS_REC,
                    None::<&str>,
                )
                .expect("Failed to mount /dev/dri");

                mount(
                    Some("/dev/input"),
                    &self.root_dir.join("dev/input"),
                    None::<&str>,
                    MsFlags::MS_BIND | MsFlags::MS_REC,
                    None::<&str>,
                )
                .expect("Failed to mount /dev/input");

                mount(
                    Some("/dev/null"),
                    &self.root_dir.join("dev/null"),
                    None::<&str>,
                    MsFlags::MS_BIND,
                    None::<&str>,
                )
                .unwrap();

                mount(
                    Some("/dev/random"),
                    &self.root_dir.join("dev/random"),
                    None::<&str>,
                    MsFlags::MS_BIND,
                    None::<&str>,
                )
                .unwrap();

                mount(
                    Some("/dev/snd"),
                    &self.root_dir.join("dev/snd"),
                    None::<&str>,
                    MsFlags::MS_BIND | MsFlags::MS_REC,
                    None::<&str>,
                )
                .expect("Failed to mount /dev/snd");

                mount(
                    Some("/dev/urandom"),
                    &self.root_dir.join("dev/urandom"),
                    None::<&str>,
                    MsFlags::MS_BIND,
                    None::<&str>,
                )
                .unwrap();

                if self.wayland {
                    mount(
                        Some("/run/user/1000/wayland-1"),
                        &run_user_dir.join("wayland-1"),
                        None::<&str>,
                        MsFlags::MS_BIND,
                        None::<&str>,
                    )
                    .unwrap();
                }

                mount(
                    Some("/run/user/1000/pipewire-0"),
                    &run_user_dir.join("pipewire-0"),
                    None::<&str>,
                    MsFlags::MS_BIND,
                    None::<&str>,
                )
                .unwrap();

                fs::write(
                    cgroup_path.join("cgroup.procs"),
                    std::process::id().to_string(),
                )
                .unwrap();

                unistd::chroot(&self.root_dir).unwrap();
                unistd::chdir(&self.work_dir).unwrap();
                unistd::sethostname(&self.hostname).unwrap();
                unistd::setgid(self.gid).unwrap();
                unistd::setuid(self.uid).unwrap();

                let command = CString::new(self.command.as_str()).unwrap();

                let args = self
                    .args
                    .iter()
                    .map(|arg| CString::new(arg.as_str()).unwrap())
                    .collect::<Vec<_>>();

                self.env.push((
                    String::from("XDG_RUNTIME_DIR"),
                    format!("/run/user/{}", self.uid),
                ));
                self.env
                    .push((String::from("HOME"), format!("/home/{}", self.username)));
                if self.wayland {
                    self.env
                        .push((String::from("WAYLAND_DISPLAY"), String::from("wayland-1")));
                }
                let env = self
                    .env
                    .iter()
                    .map(|(key, value)| CString::new(format!("{key}={value}")).unwrap())
                    .collect::<Vec<_>>();

                unistd::execve(&command, &args, &env).unwrap();
                unreachable!();
            }
            Err(_) => todo!(),
        }
    }
}
