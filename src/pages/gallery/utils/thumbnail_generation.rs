use std::{
    fs,
    path::{Path, PathBuf},
};

use iced::{
    advanced::graphics::image::image_rs,
    futures::{SinkExt, Stream},
    stream::try_channel,
};

use crate::pages::gallery::page::{THUMBNAIL_FOLDER_NAME, THUMBNAIL_SIZE};

use super::common::PhotoProcessingProgress;

pub fn generate_thumbnails(
    image_paths_to_process: Vec<PathBuf>,
) -> impl Stream<Item = Result<PhotoProcessingProgress, iced::Error>> {
    try_channel(1, move |mut progress_percentage_output| async move {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            if tx.send(0.0).await.is_err() {
                return;
            }
            let total_number_of_images = image_paths_to_process.len();
            for (image_index, image_path) in image_paths_to_process
                .into_iter()
                .filter(|image_path| image_path.file_name().is_some())
                .enumerate()
            {
                let file_name = image_path.file_name().expect("Already checked");
                let mut thumbnail_path =
                    image_path.parent().unwrap_or(Path::new("/")).to_path_buf();
                thumbnail_path.push(THUMBNAIL_FOLDER_NAME);
                if !thumbnail_path.exists() {
                    let _ = fs::create_dir_all(&thumbnail_path);
                }
                thumbnail_path.push(file_name);
                if !thumbnail_path.exists() {
                    if let Ok(img) = image_rs::open(&image_path) {
                        let original_height = img.height();
                        let original_width = img.width();

                        let new_width;
                        let new_height;
                        let x_val;
                        let y_val;
                        if original_height > original_width {
                            new_width = original_width;
                            new_height = original_width;
                            x_val = 0;
                            y_val = (original_height / 2) - (original_width / 2);
                        } else {
                            new_width = original_height;
                            new_height = original_height;
                            x_val = (original_width / 2) - (original_height / 2);
                            y_val = 0;
                        }
                        let cropped = img.crop_imm(x_val, y_val, new_width, new_height);
                        let resized = cropped.resize(
                            THUMBNAIL_SIZE,
                            THUMBNAIL_SIZE,
                            image_rs::imageops::FilterType::Nearest,
                        );
                        let _ = resized.save(&thumbnail_path);
                    };
                }
                if image_index % 10 == 0
                    && tx
                        .send(image_index as f32 / total_number_of_images as f32)
                        .await
                        .is_err()
                {
                    return;
                }
            }
            let _ = tx.send(1.0).await;
        });

        while let Some(received) = rx.recv().await {
            let _ = progress_percentage_output
                .send(PhotoProcessingProgress::ThumbnailGeneration(
                    received * 100.0,
                ))
                .await;
            if received >= 1.0 {
                let _ = progress_percentage_output
                    .send(PhotoProcessingProgress::None)
                    .await;
                break;
            }
        }
        let _ = progress_percentage_output
            .send(PhotoProcessingProgress::None)
            .await;

        Ok(())
    })
}
