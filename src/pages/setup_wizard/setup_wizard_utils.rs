use std::{
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use iced::{
    futures::{SinkExt, Stream, StreamExt},
    stream::try_channel,
};
use regex::Regex;
use tokio::{
    fs::{self, File},
    io::{AsyncBufRead, AsyncRead, AsyncWriteExt},
};

use crate::constants::APP_ID;

use super::{constants::APP_DATA_URLS, page::SetupProgressBarValue};

pub fn download_extra_files() -> impl Stream<Item = Result<SetupProgressBarValue, String>> {
    try_channel(1, move |mut output| async move {
        let total = APP_DATA_URLS.len();
        let mut downloaded = 0;
        let idirfein_data_dir = dirs::data_dir()
            .expect("Can't find data dir")
            .as_path()
            .join(APP_ID);

        for (download_url, filename) in APP_DATA_URLS {
            println!("downloading {download_url} to {filename}");
            let destination_path = idirfein_data_dir.join(filename);
            let response = reqwest::get(download_url)
                .await
                .map_err(|err| err.to_string())?;
            fs::create_dir_all(destination_path.parent().unwrap_or(Path::new("/")))
                .await
                .map_err(|err| err.to_string())?;

            let mut dest_file = File::create(destination_path)
                .await
                .map_err(|err| err.to_string())?;
            let mut byte_stream = response.bytes_stream();

            while let Some(next_bytes) = byte_stream.next().await {
                let bytes = next_bytes.map_err(|err| err.to_string())?;
                dest_file
                    .write_all(&bytes)
                    .await
                    .map_err(|err| err.to_string())?;
            }
            downloaded += 1;
            let _ = output
                .send(SetupProgressBarValue::DownloadingFile(
                    100.0 * downloaded as f32 / total as f32,
                ))
                .await;
        }

        let _ = output.send(SetupProgressBarValue::Finished).await;

        Ok(())
    })
}

struct ReadTracker<R> {
    inner: R,
    pub bytes_read: u64,
}

impl<R: AsyncRead + Unpin> AsyncRead for ReadTracker<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        let poll = std::pin::Pin::new(&mut this.inner).poll_read(cx, buf);
        if let std::task::Poll::Ready(Ok(())) = &poll {
            this.bytes_read += (buf.filled().len() - before) as u64;
        }
        poll
    }
}

impl<R: AsyncBufRead + Unpin> AsyncBufRead for ReadTracker<R> {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<&[u8]>> {
        let this = self.get_mut();
        Pin::new(&mut this.inner).poll_fill_buf(cx)
    }

    fn consume(self: Pin<&mut Self>, amount: usize) {
        let this = self.get_mut();
        Pin::new(&mut this.inner).consume(amount);
        this.bytes_read += amount as u64;
    }
}

pub fn is_valid_ip(ip: &str) -> bool {
    let re = Regex::new(r"^(\d{1,3}\.){3}\d{1,3}$").unwrap();
    if !re.is_match(ip) {
        return false;
    }

    ip.split('.').all(|part| part.parse::<u8>().is_ok())
}
