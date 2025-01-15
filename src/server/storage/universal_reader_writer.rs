use tokio::{fs::File, io::{AsyncBufRead, AsyncSeek, AsyncWrite, BufStream}};

pub trait UniversalReaderWriter: AsyncWrite + AsyncBufRead + AsyncSeek + Unpin {}

impl<T: UniversalReaderWriter> UniversalReaderWriter for BufStream<T> {}
impl UniversalReaderWriter for BufStream<File> {}
