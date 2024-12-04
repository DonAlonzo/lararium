use crate::prelude::*;
use nix::mount::{mount, umount, MsFlags};
use nix::sched::{self, CloneFlags};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, ForkResult, Gid, Uid};
use std::ffi::CString;
use std::path::PathBuf;
use std::process;

pub struct Container<'a> {
    pub rootfs_path: PathBuf,
    pub work_dir: PathBuf,
    pub command: &'a str,
    pub args: &'a [&'a str],
    pub env: &'a [(&'a str, &'a str)],
    pub hostname: &'a str,
}

impl Container<'_> {
    pub fn run(&self) {
        let proc_path = self.rootfs_path.join("proc");
        let root_path = self.rootfs_path.join("root");
        let tmp_path = self.rootfs_path.join("tmp");

        unistd::mkdir(
            &self.rootfs_path,
            nix::sys::stat::Mode::from_bits(0o755).unwrap(),
        )
        .ok();
        unistd::mkdir(
            &self.work_dir,
            nix::sys::stat::Mode::from_bits(0o755).unwrap(),
        )
        .ok();
        unistd::mkdir(&proc_path, nix::sys::stat::Mode::from_bits(0o755).unwrap()).ok();
        unistd::mkdir(&root_path, nix::sys::stat::Mode::from_bits(0o755).unwrap()).ok();
        unistd::mkdir(&tmp_path, nix::sys::stat::Mode::from_bits(0o755).unwrap()).ok();

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

        match unsafe { unistd::fork() } {
            Ok(ForkResult::Parent { child }) => {
                match waitpid(child, None).unwrap() {
                    WaitStatus::Exited(_, status) => {
                        if status != 0 {
                            error!("Container exited with status {status}");
                        } else {
                            info!("Container exited successfully.");
                        }
                    }
                    WaitStatus::Signaled(_, _, _) => error!("Container killed by signal"),
                    _ => error!("Container exited with unknown status"),
                };
                if let Err(error) = umount(&proc_path) {
                    error!("Failed to unmount /proc: {error}");
                }
                if let Err(error) = umount(&tmp_path) {
                    error!("Failed to unmount /tmp: {error}");
                }
            }
            Ok(ForkResult::Child) => {
                self.launch_container();
            }
            Err(_) => process::exit(1),
        }
    }

    fn launch_container(&self) {
        sched::unshare(
            CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWUTS,
        )
        .unwrap();

        unistd::chroot(&self.rootfs_path).unwrap();
        unistd::chdir(&self.work_dir).unwrap();
        unistd::setgid(Gid::from_raw(65534)).unwrap();
        unistd::setuid(Uid::from_raw(65534)).unwrap();
        unistd::sethostname(self.hostname).unwrap();

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
}
