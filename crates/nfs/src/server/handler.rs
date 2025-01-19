use crate::protocol::*;

pub trait Handler {
    fn access<'a>(
        &self,
        file_handle: FileHandle<'a>,
        flags: AccessFlags,
    ) -> impl std::future::Future<Output = Result<AccessResult, Error>> + Send;

    fn lookup<'a>(
        &self,
        file_handle: FileHandle<'a>,
        name: Component<'a>,
    ) -> impl std::future::Future<Output = Result<FileHandle<'a>, Error>> + Send;

    fn get_attributes<'a>(
        &self,
        file_handle: FileHandle<'a>,
        mask: AttributeMask<'a>,
    ) -> impl std::future::Future<Output = Result<Vec<AttributeValue<'a>>, Error>> + Send;

    fn read_directory<'a>(
        &self,
        file_handle: FileHandle<'a>,
        args: ReadDirectoryArgs<'a>,
    ) -> impl std::future::Future<Output = Result<ReadDirectoryResult<'a>, Error>> + Send;

    fn destroy_session(
        &self,
        session_id: SessionId,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;

    fn destroy_client_id(
        &self,
        client_id: ClientId,
    ) -> impl std::future::Future<Output = Result<(), Error>> + Send;
}
