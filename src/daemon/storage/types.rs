use tokio::io::{AsyncBufRead, AsyncSeek, AsyncWrite};

pub trait AsyncReadWrite: AsyncWrite + AsyncBufRead + AsyncSeek + Unpin {}

impl<T: AsyncWrite + AsyncBufRead + AsyncSeek + Unpin> AsyncReadWrite for T {}
