use std::{
    path::{Path, PathBuf},
    pin::Pin,
    process::Command,
    task::{Context, Poll},
};

use async_compression::tokio::bufread::XzDecoder;
use iced::{
    futures::{SinkExt, Stream, StreamExt},
    stream::try_channel,
};
use tokio::{
    fs::{self, File},
    io::{AsyncBufRead, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

use crate::constants::APP_ID;

use super::{page::SetupProgressBarValue, APP_DATA_URLS};

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

pub fn download_file(
    url: String,
    destination_path: PathBuf,
) -> impl Stream<Item = Result<SetupProgressBarValue, String>> {
    try_channel(1, move |mut output| async move {
        let response = reqwest::get(url).await.map_err(|err| err.to_string())?;
        let total = response
            .content_length()
            .ok_or(String::from("No Content Length Header found"))?;

        let _ = output
            .send(SetupProgressBarValue::DownloadingFile(0.0))
            .await;

        fs::create_dir_all(destination_path.parent().unwrap_or(Path::new("/")))
            .await
            .map_err(|err| err.to_string())?;

        let mut dest_file = File::create(destination_path)
            .await
            .map_err(|err| err.to_string())?;
        let mut byte_stream = response.bytes_stream();
        let mut downloaded = 0;

        while let Some(next_bytes) = byte_stream.next().await {
            let bytes = next_bytes.map_err(|err| err.to_string())?;
            downloaded += bytes.len();
            dest_file
                .write_all(&bytes)
                .await
                .map_err(|err| err.to_string())?;

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

pub fn extract_img(
    img_archive_path: PathBuf,
    extracted_img_file_path: PathBuf,
) -> impl Stream<Item = Result<SetupProgressBarValue, String>> {
    try_channel(1, move |mut output| async move {
        let _ = output.send(SetupProgressBarValue::ExtractingImg(0.0)).await;

        fs::create_dir_all(img_archive_path.parent().unwrap_or(Path::new("/")))
            .await
            .map_err(|err| err.to_string())?;

        let img_file = File::open(&img_archive_path)
            .await
            .map_err(|err| err.to_string())?;
        let total_bytes = img_file
            .metadata()
            .await
            .map_err(|err| err.to_string())?
            .len();
        let buffered_img_file = BufReader::new(img_file);
        let tracked_buffered_img_file = ReadTracker {
            inner: buffered_img_file,
            bytes_read: 0,
        };
        let mut decompressor = XzDecoder::new(tracked_buffered_img_file);

        let extracted_file = File::create(extracted_img_file_path)
            .await
            .map_err(|err| err.to_string())?;
        let mut buffered_extracted_file = BufWriter::new(extracted_file);

        let mut buffer = vec![0u8; bytesize::MB as usize];

        loop {
            match decompressor
                .read(&mut buffer)
                .await
                .map_err(|err| err.to_string())?
            {
                0 => break, // EOF
                n => {
                    buffered_extracted_file
                        .write_all(&buffer[..n])
                        .await
                        .unwrap();

                    let _ = output
                        .send(SetupProgressBarValue::ExtractingImg(
                            100.0 * decompressor.get_ref().bytes_read as f32 / total_bytes as f32,
                        ))
                        .await;
                }
            }
        }

        let _ = output.send(SetupProgressBarValue::Finished).await;

        Ok(())
    })
}

pub fn flash_img_to_sd_card(extracted_img_file_path: PathBuf) {
    println!(
        "{:?}",
        Command::new("flatpak-spawn")
            .arg("--host")
            .arg("pkexec")
            .arg("ls")
            .output()
    );
}
