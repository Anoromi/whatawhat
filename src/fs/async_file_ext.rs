use anyhow::Result;
use tokio::{fs::File, task::spawn_blocking};


trait AsyncFileExt {
    async fn lock_exclusive_async(self) -> Result<File>;
    async fn lock_shared_async(self) -> Result<File>;
}

impl AsyncFileExt for File {
    async fn lock_exclusive_async(self) -> Result<File> {
        let file = spawn_blocking(move || -> Result<_> {
            fs4::tokio::AsyncFileExt::lock_exclusive(&self)?;
            Ok(self)
        })
        .await??;

        Ok(file)
    }

    async fn lock_shared_async(self) -> Result<File> {
        let file = spawn_blocking(move || -> Result<_> {
            fs4::tokio::AsyncFileExt::lock_exclusive(&self)?;
            Ok(self)
        })
        .await??;

        Ok(file)
    }
}
