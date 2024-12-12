use crate::prelude::*;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::mount::{mount, umount, MsFlags};
use nix::sched::{self, CloneFlags};
use nix::sys::signal::{kill, Signal};
use nix::sys::stat::{fchmod, Mode};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, dup2, fchown, pipe, ForkResult, Gid, Uid};
use std::ffi::CString;
use std::fmt;
use std::fs;
use std::os::fd::IntoRawFd;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct ContainerBlueprint {
    pub rootfs_path: PathBuf,
    pub work_dir: PathBuf,
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub hostname: String,
    pub username: String,
    pub gid: u32,
    pub uid: u32,
}

pub struct ContainerHandle {
    signal_tx: Option<oneshot::Sender<Signal>>,
}

pub struct ImageUri<'a> {
    registry: &'a str,
    repository: &'a str,
    image: &'a str,
    tag: &'a str,
    arch: &'a str,
}

pub struct ImageCache {
    path: Path,
}

#[derive(Debug)]
pub enum Error {}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(f, "Error")
    }
}

impl ImageCache {
    pub fn download(
        &self,
        uri: ImageUri,
        dst: &Path,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl ContainerBlueprint {
    pub fn run(&self) -> Result<ContainerHandle, Error> {
        let rootfs_path = self.rootfs_path.clone();
        let work_dir = self.work_dir.clone();
        let hostname = self.hostname.clone();
        let username = self.username.clone();
        let command = self.command.clone();
        let args = self.args.clone();
        let mut env = self.env.clone();
        let gid = Gid::from_raw(self.gid);
        let uid = Uid::from_raw(self.uid);
        let (signal_tx, mut signal_rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
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

            let cgroup_path = Path::new("/sys/fs/cgroup/lararium").join(&hostname);
            fs::create_dir_all(&cgroup_path).unwrap();

            fs::create_dir_all(rootfs_path.join(&work_dir)).unwrap();
            fs::create_dir_all(rootfs_path.join("proc")).unwrap();
            fs::create_dir_all(rootfs_path.join("root")).unwrap();
            fs::create_dir_all(rootfs_path.join("sys")).unwrap();
            fs::create_dir_all(rootfs_path.join("tmp")).unwrap();
            fs::create_dir_all(rootfs_path.join("dev/dri")).unwrap();
            fs::create_dir_all(rootfs_path.join("dev/input")).unwrap();
            fs::File::create(rootfs_path.join("dev/null")).unwrap();
            fs::create_dir_all(rootfs_path.join("dev/snd")).unwrap();

            let run_user_dir = rootfs_path.join("run/user").join(uid.to_string());
            fs::create_dir_all(&run_user_dir).unwrap();
            {
                let file = fs::File::open(&run_user_dir).unwrap();
                fchown(file.as_raw_fd(), Some(uid), Some(gid)).unwrap();
                fchmod(file.as_raw_fd(), Mode::from_bits(0o700).unwrap()).unwrap();
            }

            let wayland_socket = run_user_dir.join("wayland-1");
            fs::File::create(&wayland_socket).unwrap();
            {
                let file = fs::File::open(wayland_socket).unwrap();
                fchown(file.as_raw_fd(), Some(uid), Some(gid)).unwrap();
                fchmod(file.as_raw_fd(), Mode::from_bits(0o700).unwrap()).unwrap();
            }

            let pipewire_socket = run_user_dir.join("pipewire-0");
            fs::File::create(&pipewire_socket).unwrap();
            {
                let file = fs::File::open(pipewire_socket).unwrap();
                fchown(file.as_raw_fd(), Some(uid), Some(gid)).unwrap();
                fchmod(file.as_raw_fd(), Mode::from_bits(0o700).unwrap()).unwrap();
            }

            let home_dir = rootfs_path.join("home").join(&username);
            fs::create_dir_all(&home_dir).unwrap();
            {
                let file = fs::File::open(home_dir).unwrap();
                fchown(file.as_raw_fd(), Some(uid), Some(gid)).unwrap();
                fchmod(file.as_raw_fd(), Mode::from_bits(0o751).unwrap()).unwrap();
            }

            fs::write(rootfs_path.join("etc/hostname"), format!("{hostname}\n")).unwrap();
            fs::write(
                rootfs_path.join("etc/group"),
                format!("root:x:0:\n{username}:x:{gid}:\n"),
            )
            .unwrap();
            fs::write(
                rootfs_path.join("etc/passwd"),
                format!("root:x:0:0:root:/root:/bin/sh\n{username}:x:{uid}:{gid}:{username}:/home/{username}:/bin/sh\n"),
            ).unwrap();
            fs::write(
                rootfs_path.join("etc/resolv.conf"),
                format!("nameserver 127.0.0.1"),
            )
            .unwrap();

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

                        if let Ok(signal) = signal_rx.try_recv() {
                            if let Err(error) = kill(child, signal) {
                                error!("Failed to kill child process: {error}");
                            } else {
                                info!("Killed container with signal {signal}");
                            }
                        }

                        match waitpid(child, Some(nix::sys::wait::WaitPidFlag::WNOHANG)).unwrap() {
                            WaitStatus::StillAlive | WaitStatus::Continued(_) => {}
                            WaitStatus::Exited(_, status) => {
                                if status != 0 {
                                    error!("Container exited with status {status}");
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
                    if let Err(error) = umount(&rootfs_path.join("proc")) {
                        error!("Failed to unmount /proc: {error}");
                    }
                    if let Err(error) = umount(&rootfs_path.join("sys")) {
                        error!("Failed to unmount /sys: {error}");
                    }
                    if let Err(error) = umount(&rootfs_path.join("tmp")) {
                        error!("Failed to unmount /tmp: {error}");
                    }
                    if let Err(error) = umount(&rootfs_path.join("dev/dri")) {
                        error!("Failed to unmount /dev/dri: {error}");
                    }
                    if let Err(error) = umount(&rootfs_path.join("dev/input")) {
                        error!("Failed to unmount /dev/input: {error}");
                    }
                    if let Err(error) = umount(&rootfs_path.join("dev/null")) {
                        error!("Failed to unmount /dev/null: {error}");
                    }
                    if let Err(error) = umount(&rootfs_path.join("dev/snd")) {
                        error!("Failed to unmount /dev/snd: {error}");
                    }
                    if let Err(error) = umount(&run_user_dir.join("wayland-1")) {
                        error!("Failed to unmount wayland socket: {error}");
                    }
                    if let Err(error) = umount(&run_user_dir.join("pipewire-0")) {
                        error!("Failed to unmount pipewire socket: {error}");
                    }
                }
                Ok(ForkResult::Child) => {
                    drop(stdout_read);
                    drop(stderr_read);
                    dup2(stdout_write.into_raw_fd(), 1).unwrap();
                    dup2(stderr_write.into_raw_fd(), 2).unwrap();

                    mount(
                        Some("proc"),
                        &rootfs_path.join("proc"),
                        Some("proc"),
                        MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
                        None::<&str>,
                    )
                    .unwrap();

                    mount(
                        Some("sysfs"),
                        &rootfs_path.join("sys"),
                        Some("sysfs"),
                        MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
                        None::<&str>,
                    )
                    .unwrap();

                    mount(
                        Some("tmpfs"),
                        &rootfs_path.join("tmp"),
                        Some("tmpfs"),
                        MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
                        None::<&str>,
                    )
                    .unwrap();

                    mount(
                        Some("/dev/dri"),
                        &rootfs_path.join("dev/dri"),
                        None::<&str>,
                        MsFlags::MS_BIND | MsFlags::MS_REC,
                        None::<&str>,
                    )
                    .expect("Failed to mount /dev/dri");

                    mount(
                        Some("/dev/input"),
                        &rootfs_path.join("dev/input"),
                        None::<&str>,
                        MsFlags::MS_BIND | MsFlags::MS_REC,
                        None::<&str>,
                    )
                    .expect("Failed to mount /dev/input");

                    mount(
                        Some("/dev/null"),
                        &rootfs_path.join("dev/null"),
                        None::<&str>,
                        MsFlags::MS_BIND,
                        None::<&str>,
                    )
                    .unwrap();

                    mount(
                        Some("/dev/snd"),
                        &rootfs_path.join("dev/snd"),
                        None::<&str>,
                        MsFlags::MS_BIND | MsFlags::MS_REC,
                        None::<&str>,
                    )
                    .expect("Failed to mount /dev/snd");

                    mount(
                        Some("/run/user/1000/wayland-1"),
                        &run_user_dir.join("wayland-1"),
                        None::<&str>,
                        MsFlags::MS_BIND,
                        None::<&str>,
                    )
                    .unwrap();

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

                    unistd::chroot(&rootfs_path).unwrap();
                    unistd::chdir(&work_dir).unwrap();
                    unistd::sethostname(&hostname).unwrap();
                    unistd::setgid(gid).unwrap();
                    unistd::setuid(uid).unwrap();

                    let command = CString::new(command.as_str()).unwrap();

                    let args = args
                        .iter()
                        .map(|arg| CString::new(arg.as_str()).unwrap())
                        .collect::<Vec<_>>();

                    env.push((String::from("XDG_RUNTIME_DIR"), format!("/run/user/{uid}")));
                    env.push((String::from("HOME"), format!("/home/{username}")));
                    env.push((String::from("WAYLAND_DISPLAY"), String::from("wayland-1")));
                    let env = env
                        .iter()
                        .map(|(key, value)| CString::new(format!("{key}={value}")).unwrap())
                        .collect::<Vec<_>>();

                    unistd::execve(&command, &args, &env).unwrap();
                }
                Err(_) => process::exit(1),
            }
        });

        Ok(ContainerHandle {
            signal_tx: Some(signal_tx),
        })
    }
}

impl Drop for ContainerHandle {
    fn drop(&mut self) {
        if let Some(signal_tx) = self.signal_tx.take() {
            let _ = signal_tx.send(Signal::SIGKILL);
        }
    }
}
