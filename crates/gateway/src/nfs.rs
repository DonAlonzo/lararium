use lararium_nfs::*;

impl Handler for crate::Gateway {
    async fn lookup<'a>(
        &self,
        current_fh: FileHandle<'a>,
        name: Component<'a>,
    ) -> Result<FileHandle<'a>, Error> {
        Ok(FileHandle::from(Opaque::from(name.as_bytes().to_vec())))
    }
}
