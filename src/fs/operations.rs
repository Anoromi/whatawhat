use anyhow::Result;
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, AsyncWrite};

/// Moves backwards in a file to beginning of a previous line.
/// Useful if you wan't to overwrite last line with new data.
pub async fn seek_line_backwards(
    file: &mut (impl AsyncSeek + AsyncWrite + AsyncRead + Unpin),
    buffer: &mut [u8],
) -> Result<(), io::Error> {
    // We skip first new line that is right before the buffer, so that reading doesn't get stuck.
    // For example: need_to_read_this\nwe_are_here_now\n
    let mut need_to_skip = 1usize;
    loop {
        let leftover = file.stream_position().await?;
        if leftover == 0 {
            return Ok(());
        }
        let next_chunk = u64::min(leftover, buffer.len() as u64) as usize;
        file.seek(std::io::SeekFrom::Current(-(next_chunk as i64)))
            .await?;

        file.read_exact(&mut buffer[..next_chunk]).await?;
        let iter = buffer[..next_chunk].iter().rev().enumerate();
        let iter = iter.skip(need_to_skip);
        for (index, value) in iter {
            if *value == b'\n' {
                file.seek(std::io::SeekFrom::Current(-(index as i64)))
                    .await?;
                return Ok(());
            }
        }

        need_to_skip = need_to_skip.saturating_sub(1);
        file.seek(std::io::SeekFrom::Current(-(next_chunk as i64)))
            .await?;
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use anyhow::Result;

    use tempfile::tempfile;
    use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};

    use crate::fs::operations::seek_line_backwards;

    #[tokio::test]
    async fn test_seek_line_backwards_basic() -> Result<()> {
        let mut file = tempfile()?;
        let b = "test hello theere\n\
                 test hello theere\n\
                 how do you do";

        file.write_all(b.as_bytes())?;

        let mut file = tokio::fs::File::from_std(file);

        seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice())
            .await
            .unwrap();

        seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice())
            .await
            .unwrap();

        seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice())
            .await
            .unwrap();

        assert_eq!(file.stream_position().await?, 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_seek_line_backwards_empty() -> Result<()> {
        let file = tempfile()?;
        let file = tokio::fs::File::from_std(file);

        let mut file = BufReader::new(file);
        let mut value = String::new();
        file.read_line(&mut value).await?;

        seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice()).await?;

        assert_eq!(file.stream_position().await?, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_seek_line_backwards_reversability() -> Result<()> {
        let mut file = tempfile()?;
        let b = "test hello theere\n\
                 test hello theere\n\
                 how do you do";

        let positions = b
            .bytes()
            .enumerate()
            .filter(|v| v.1 == b'\n')
            .map(|v| v.0 + 1)
            .collect::<Vec<_>>();

        file.write_all(b.as_bytes())?;

        let mut file = BufReader::new(tokio::fs::File::from_std(file));

        file.seek(std::io::SeekFrom::Start(0)).await?;

        {
            file.read_line(&mut String::new()).await?;

            seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice())
                .await
                .unwrap();
        }

        assert_eq!(file.stream_position().await?, 0);

        {
            file.read_line(&mut String::new()).await?;
            file.read_line(&mut String::new()).await?;

            seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice())
                .await
                .unwrap();

            assert_eq!(file.stream_position().await?, positions[0] as u64);

            seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice())
                .await
                .unwrap();

            assert_eq!(file.stream_position().await?, 0);
        }

        {
            file.read_line(&mut String::new()).await?;
            file.read_line(&mut String::new()).await?;
            file.read_line(&mut String::new()).await?;

            seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice())
                .await
                .unwrap();

            assert_eq!(file.stream_position().await?, positions[1] as u64);

            seek_line_backwards(&mut file, vec![0; 1024].as_mut_slice())
                .await
                .unwrap();

            assert_eq!(file.stream_position().await?, positions[0] as u64);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_seek_line_backwards_small_buffer() -> Result<()> {
        let mut file = tempfile()?;
        let b = "test hello theere\n\
                 test hello theere\n\
                 how do you do";

        let positions = b
            .bytes()
            .enumerate()
            .filter(|v| v.1 == b'\n')
            .map(|v| v.0 + 1)
            .collect::<Vec<_>>();

        file.write_all(b.as_bytes())?;

        let mut file = BufReader::new(tokio::fs::File::from_std(file));

        file.seek(std::io::SeekFrom::Start(0)).await?;

        {
            file.read_line(&mut String::new()).await?;

            seek_line_backwards(&mut file, vec![0; 2].as_mut_slice())
                .await
                .unwrap();
        }

        assert_eq!(file.stream_position().await?, 0);

        {
            file.read_line(&mut String::new()).await?;
            file.read_line(&mut String::new()).await?;

            seek_line_backwards(&mut file, vec![0; 2].as_mut_slice())
                .await
                .unwrap();

            assert_eq!(file.stream_position().await?, positions[0] as u64);

            seek_line_backwards(&mut file, vec![0; 2].as_mut_slice())
                .await
                .unwrap();

            assert_eq!(file.stream_position().await?, 0);
        }

        {
            file.read_line(&mut String::new()).await?;
            file.read_line(&mut String::new()).await?;
            file.read_line(&mut String::new()).await?;

            seek_line_backwards(&mut file, vec![0; 2].as_mut_slice())
                .await
                .unwrap();

            assert_eq!(file.stream_position().await?, positions[1] as u64);

            seek_line_backwards(&mut file, vec![0; 2].as_mut_slice())
                .await
                .unwrap();

            assert_eq!(file.stream_position().await?, positions[0] as u64);
        }

        Ok(())
    }
}
