use crate::prelude::*;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::mount::{mount, umount, MsFlags};
use nix::sched::{self, CloneFlags};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, dup2, pipe, ForkResult, Gid, Uid};
use std::ffi::CString;
use std::fs;
use std::os::fd::IntoRawFd;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process;

pub struct Container<'a> {
    pub rootfs_path: PathBuf,
    pub work_dir: PathBuf,
    pub command: &'a str,
    pub args: &'a [&'a str],
    pub env: &'a [(&'a str, &'a str)],
    pub hostname: &'a str,
    pub gid: u32,
    pub uid: u32,
}

impl Container<'_> {
    pub fn run(&self) {
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

        let cgroup_path = Path::new("/sys/fs/cgroup/lararium/").join(self.hostname);
        fs::create_dir_all(&cgroup_path).unwrap();

        let proc_path = self.rootfs_path.join("proc");
        let root_path = self.rootfs_path.join("root");
        let tmp_path = self.rootfs_path.join("tmp");

        fs::create_dir_all(&self.work_dir).unwrap();
        fs::create_dir_all(&proc_path).unwrap();
        fs::create_dir_all(&root_path).unwrap();
        fs::create_dir_all(&tmp_path).unwrap();

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
                if let Err(error) = umount(&proc_path) {
                    error!("Failed to unmount /proc: {error}");
                }
                if let Err(error) = umount(&tmp_path) {
                    error!("Failed to unmount /tmp: {error}");
                }
            }
            Ok(ForkResult::Child) => {
                drop(stdout_read);
                drop(stderr_read);
                dup2(stdout_write.into_raw_fd(), 1).unwrap();
                dup2(stderr_write.into_raw_fd(), 2).unwrap();

                mount(
                    Some("proc"),
                    &proc_path,
                    Some("proc"),
                    MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC | MsFlags::MS_NODEV,
                    None::<&str>,
                )
                .unwrap();

                mount(
                    Some("tmpfs"),
                    &tmp_path,
                    Some("tmpfs"),
                    MsFlags::MS_NOEXEC | MsFlags::MS_NOSUID | MsFlags::MS_NODEV,
                    None::<&str>,
                )
                .unwrap();

                fs::write(
                    cgroup_path.join("cgroup.procs"),
                    std::process::id().to_string(),
                )
                .unwrap();

                unistd::chroot(&self.rootfs_path).unwrap();
                unistd::chdir(&self.work_dir).unwrap();
                unistd::sethostname(self.hostname).unwrap();
                unistd::setgid(Gid::from_raw(self.gid)).unwrap();
                unistd::setuid(Uid::from_raw(self.uid)).unwrap();

                let command = CString::new(self.command).unwrap();

                let args = self
                    .args
                    .iter()
                    .map(|&arg| CString::new(arg).unwrap())
                    .collect::<Vec<_>>();

                let env = self
                    .env
                    .iter()
                    .map(|&(key, value)| CString::new(format!("{key}={value}")).unwrap())
                    .collect::<Vec<_>>();

                unistd::execve(&command, &args, &env).unwrap();
            }
            Err(_) => process::exit(1),
        }
    }
}
