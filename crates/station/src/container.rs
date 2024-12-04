use crate::prelude::*;
use nix::mount::{mount, umount, MsFlags};
use nix::sched::{self, CloneFlags};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{self, ForkResult, Gid, Uid};
use std::ffi::CString;
use std::fs;
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

                self.launch_container(&cgroup_path);
            }
            Err(_) => process::exit(1),
        }
    }

    fn launch_container(
        &self,
        cgroup_path: &Path,
    ) {
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
}
