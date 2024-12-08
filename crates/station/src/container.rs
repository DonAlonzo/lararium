use crate::prelude::*;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::mount::{mount, umount, MsFlags};
use nix::sched::{self, CloneFlags};
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, chown, dup2, pipe, ForkResult, Gid, Uid};
use std::ffi::CString;
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
    pub gid: u32,
    pub uid: u32,
}

pub struct ContainerHandle {
    signal_tx: Option<oneshot::Sender<Signal>>,
}

impl ContainerBlueprint {
    pub fn run(&self) -> ContainerHandle {
        let rootfs_path = self.rootfs_path.clone();
        let work_dir = self.work_dir.clone();
        let hostname = self.hostname.clone();
        let command = self.command.clone();
        let args = self.args.clone();
        let env = self.env.clone();
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

            let cgroup_path = Path::new("/sys/fs/cgroup/lararium/").join(&hostname);
            fs::create_dir_all(&cgroup_path).unwrap();

            fs::create_dir_all(rootfs_path.join(&work_dir)).unwrap();
            fs::create_dir_all(rootfs_path.join("proc")).unwrap();
            fs::create_dir_all(rootfs_path.join("root")).unwrap();
            fs::create_dir_all(rootfs_path.join("tmp")).unwrap();
            fs::create_dir_all(rootfs_path.join("dev/dri")).unwrap();
            fs::File::create(rootfs_path.join("dev/null")).unwrap();
            fs::create_dir_all(rootfs_path.join("home/donalonzo")).unwrap();
            chown(&rootfs_path.join("home/donalonzo"), Some(uid), Some(gid)).unwrap();

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
                                error!("Container killed by signal");
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
                    if let Err(error) = umount(&rootfs_path.join("tmp")) {
                        error!("Failed to unmount /tmp: {error}");
                    }
                    if let Err(error) = umount(&rootfs_path.join("dev/null")) {
                        error!("Failed to unmount /dev/null: {error}");
                    }
                    if let Err(error) = umount(&rootfs_path.join("dev/dri")) {
                        error!("Failed to unmount /dev/dri: {error}");
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
                        Some("tmpfs"),
                        &rootfs_path.join("tmp"),
                        Some("tmpfs"),
                        MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
                        None::<&str>,
                    )
                    .unwrap();

                    mount(
                        Some("/dev/null"),
                        &rootfs_path.join("dev/null"),
                        None::<&str>,
                        MsFlags::MS_BIND,
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

                    let env = env
                        .iter()
                        .map(|(key, value)| CString::new(format!("{key}={value}")).unwrap())
                        .collect::<Vec<_>>();

                    unistd::execve(&command, &args, &env).unwrap();
                }
                Err(_) => process::exit(1),
            }
        });
        ContainerHandle {
            signal_tx: Some(signal_tx),
        }
    }
}

impl Drop for ContainerHandle {
    fn drop(&mut self) {
        if let Some(signal_tx) = self.signal_tx.take() {
            let _ = signal_tx.send(Signal::SIGKILL);
        }
    }
}
