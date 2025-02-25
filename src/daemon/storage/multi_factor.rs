use std::{
    io::Cursor,
    marker::PhantomData,
    sync::{Arc},
};

use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{io::{
    AsyncReadExt, AsyncSeekExt, AsyncWriteExt,
}, sync::Mutex};

use super::types::AsyncReadWrite;

pub trait Repository<T> {
    fn get_all(&self) -> impl std::future::Future<Output = Result<Vec<T>>>;

    fn purge_records(&self) -> impl std::future::Future<Output = Result<()>>;

    // Might be used in the future for volatile + non-volatile storages
    fn persist(&self) -> impl std::future::Future<Output = Result<()>>;

    fn add_event(&self, value: T) -> impl std::future::Future<Output = Result<()>>;
}

struct InternalStorage<T, R> {
    phantom: PhantomData<T>,
    file_handle: R,
}

impl<T: Serialize + DeserializeOwned + Clone, R: AsyncReadWrite> InternalStorage<T, R> {
    async fn read_value(&mut self) -> Result<T> {
        let mut buf = Vec::with_capacity(1024);
        self.file_handle.read_to_end(&mut buf).await?;
        self.file_handle.seek(std::io::SeekFrom::Start(0)).await?;
        Ok(serde_json::from_slice::<T>(&buf)?)
    }

    async fn write_value(&mut self, value: T) -> Result<()> {
        let mut v = Cursor::new(serde_json::to_vec(&value)?);
        self.file_handle.write_all_buf(&mut v).await?;
        Ok(())
    }
}

pub struct SafeRepositoryImpl<T, R> {
    internal: Arc<Mutex<InternalStorage<Vec<T>, R>>>,
    phantom: PhantomData<T>,
}


impl<T: Serialize + DeserializeOwned + Clone, R: AsyncReadWrite> Repository<T>
    for SafeRepositoryImpl<T, R>
{
    async fn get_all(&self) -> Result<Vec<T>> {
        let mut lock = self.internal.lock().await;
        lock.read_value().await
    }

    async fn purge_records(&self) -> Result<()> {
        let mut lock = self.internal.lock().await;
        lock.write_value(Vec::default()).await
    }

    async fn persist(&self) -> Result<()> {
        Ok(())
    }

    async fn add_event(&self, value: T) -> Result<()> {
        let mut lock = self.internal.lock().await;
        let mut vec = lock.read_value().await?;
        vec.push(value);
        lock.write_value(vec).await?;
        Ok(())
    }
}
