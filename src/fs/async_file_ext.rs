use std::{io::Cursor, pin::Pin, task::ready};

use anyhow::Result;
use tokio::{
    fs::File,
    pin,
    task::{block_in_place, spawn_blocking},
};

pub trait FileLock {
    fn lock_exclusive_in_place(&self) -> Result<()>;
    fn lock_shared_in_place(&self) -> Result<()>;
    fn unlock_in_place(&self) -> Result<()>;
}

impl FileLock for File {
    // Until I figure out how to use spawn_blocking with non static references this is the best
    // option
    fn lock_exclusive_in_place(&self) -> Result<()> {
        // block_in_place(|| -> Result<_> {
            fs4::tokio::AsyncFileExt::lock_exclusive(self)?;
        //     Ok(())
        // })?;
        Ok(())
    }

    fn lock_shared_in_place(&self) -> Result<()> {
        // block_in_place(|| -> Result<_> {
            fs4::tokio::AsyncFileExt::lock_shared(self)?;
        //     Ok(())
        // })?;
        Ok(())
    }

    fn unlock_in_place(&self) -> Result<()> {
        // block_in_place(|| -> Result<_> {
            fs4::tokio::AsyncFileExt::unlock(self)?;
        //     Ok(())
        // })?;
        Ok(())
    }
}

impl<T> FileLock for Cursor<T> {
    fn lock_exclusive_in_place(&self) -> Result<()> {
        Ok(())
    }

    fn lock_shared_in_place(&self) -> Result<()> {
        Ok(())
    }

    fn unlock_in_place(&self) -> Result<()> {
        Ok(())
    }
}
