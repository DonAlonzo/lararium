use crate::protocol::*;

pub trait Handler {
    fn access<'a>(
        &self,
        current_fh: FileHandle<'a>,
        flags: AccessFlags,
    ) -> impl std::future::Future<Output = Result<AccessResult, Error>> + Send;

    fn lookup<'a>(
        &self,
        current_fh: FileHandle<'a>,
        name: Component<'a>,
    ) -> impl std::future::Future<Output = Result<FileHandle<'a>, Error>> + Send;
}
