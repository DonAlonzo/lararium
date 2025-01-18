use crate::protocol::*;

pub trait Handler {
    fn lookup<'a>(
        &self,
        current_fh: FileHandle<'a>,
        name: Component<'a>,
    ) -> impl std::future::Future<Output = Result<FileHandle<'a>, Error>> + Send;
}
