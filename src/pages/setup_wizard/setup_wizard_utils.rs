use std::{
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use async_compression::tokio::bufread::XzDecoder;
use iced::{
    futures::{SinkExt, Stream, StreamExt},
    stream::try_channel,
};
use regex::Regex;
use tokio::{
    fs::{self, File},
    io::{AsyncBufRead, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

use crate::{
    constants::APP_ID,
    pages::setup_wizard::update::{RPI_OS_IMAGE_ARCHIVE_FILENAME, RPI_OS_IMAGE_EXTRACTED_FILENAME},
};

use super::{
    constants::APP_DATA_URLS,
    page::{DiskInfo, ServerConfig, SetupProgressBarValue},
};

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

        let _ = output
            .send(SetupProgressBarValue::DownloadingFile(100.0))
            .await;

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

        let mut buffer = vec![0u8; 1048576]; // Number of bytes in a megabyte

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

        let _ = output
            .send(SetupProgressBarValue::ExtractingImg(100.0))
            .await;

        Ok(())
    })
}

pub fn flash_img_to_sd_card(
    extracted_img_file_path: PathBuf,
    sd_card_to_write_to: String,
    wpa_supplicant_file_content: String,
) -> impl Stream<Item = Result<SetupProgressBarValue, String>> {
    try_channel(1, move |mut output| async move {
        // TODO check statuses to see if auth wasn't given, if so throw err, view says operation was cancelled and show a restart button
        let _ = output.send(SetupProgressBarValue::FlashingSdCard).await;

        let mnt_dir = "/mnt/bootfs_rpi_idirfein";
        let boot_fs_partition_name = if sd_card_to_write_to
            .chars()
            .last()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            format!("{sd_card_to_write_to}p1")
        } else {
            format!("{sd_card_to_write_to}1")
        };

        let commands = format!(
            r#"
    dd if={0} of={1} bs=4M conv=fsync &&
    mkdir -p {mnt_dir} &&
    mount {2} {mnt_dir} &&
    echo '{3}' > {mnt_dir}/wpa_supplicant.conf &&
    umount {mnt_dir}
    "#,
            extracted_img_file_path.to_string_lossy(),
            sd_card_to_write_to,
            boot_fs_partition_name,
            wpa_supplicant_file_content.replace('\'', r"'\''"),
        );

        let combined_command = tokio::process::Command::new("pkexec")
            .arg("sh")
            .arg("-c")
            .arg(commands)
            .output();
        // let combined_command = tokio::process::Command::new("flatpak-spawn")
        //     .arg("--host")
        //     .arg("pkexec")
        //     .arg("sh")
        //     .arg("-c")
        //     .arg(commands)
        //     .output(); TODO add this back in for flatpak

        println!("Spawned combined pkexec command");
        let finished = combined_command.await.map_err(|err| err.to_string())?;
        println!(
            "Combined command output stderr: {} stdout: {}",
            String::from_utf8_lossy(&finished.stderr),
            String::from_utf8_lossy(&finished.stdout)
        );

        // Delete no longer needed img and archive
        let mut img_archive_download_file_path =
            dirs::data_local_dir().expect("No config directory, big problem");
        img_archive_download_file_path.push(APP_ID);
        img_archive_download_file_path.push(RPI_OS_IMAGE_ARCHIVE_FILENAME);
        let mut extracted_img_file_path =
            dirs::data_local_dir().expect("No config directory, big problem");
        extracted_img_file_path.push(APP_ID);
        extracted_img_file_path.push(RPI_OS_IMAGE_EXTRACTED_FILENAME);
        let _ = tokio::fs::remove_file(img_archive_download_file_path).await;
        let _ = tokio::fs::remove_file(extracted_img_file_path).await;

        let _ = output.send(SetupProgressBarValue::Finished).await;

        Ok(())
    })
}

pub fn is_valid_ip(ip: &str) -> bool {
    let re = Regex::new(r"^(\d{1,3}\.){3}\d{1,3}$").unwrap();
    if !re.is_match(ip) {
        return false;
    }

    ip.split('.').all(|part| part.parse::<u8>().is_ok())
}

pub async fn get_list_of_disks() -> Vec<DiskInfo> {
    // let list_disks_command_output = tokio::process::Command::new("flatpak-spawn")
    //     .arg("--host")
    //     .arg("lsblk")
    //     .arg("-dno")
    //     .arg("NAME,SIZE")
    //     .output()
    //     .await
    //     .unwrap(); TODO add this back in for flatpak
    let list_disks_command_output = tokio::process::Command::new("lsblk")
        .arg("-dno")
        .arg("NAME,SIZE")
        .output()
        .await
        .unwrap();
    String::from_utf8_lossy(&list_disks_command_output.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next()) {
                (Some(name), Some(size)) => Some(DiskInfo {
                    name: format!("/dev/{name}"),
                    total_space: size.to_string(),
                }),
                _ => None,
            }
        })
        .collect()
}

pub async fn write_config_to_rpi(
    sd_card_to_write_to: String,
    server_config: ServerConfig,
) -> Result<(), String> {
    let mnt_dir = "/mnt/rootfs_rpi_idirfein";
    let root_fs_partition_name = if sd_card_to_write_to
        .chars()
        .last()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("{sd_card_to_write_to}p2")
    } else {
        format!("{sd_card_to_write_to}2")
    };

    let setup_service_content = r#"[Unit]
Description=Run First Boot Script
After=network.target

[Service]
Type=oneshot
ExecStart=/etc/first_boot.sh
RemainAfterExit=true

[Install]
WantedBy=multi-user.target
"#;

    let serialised_config = serde_json::to_string(&server_config)
        .unwrap()
        .replace('\'', r"'\''");
    let server_password = server_config.server_password;
    let duckdns_domain = server_config.duckdns_domain;
    let duckdns_token = server_config.duckdns_token;
    let certbot_email = server_config.certbot_email;
    let is_lan_only = server_config.is_lan_only;
    let setup_script_content = format!(
        r#"
#!/bin/bash
MARKER_FILE="/etc/first_boot_done"
if [ -f "$MARKER_FILE" ]; then
    exit 0
fi
touch "$MARKER_FILE" # TODO remove this

sudo apt update

# Set the password for the 'pi' user
echo "pi:{server_password}" | sudo chpasswd

# Automatically run the binary 'idirfein_server' on startup
echo "@reboot /home/pi/idirfein_server" | crontab -

# Automatically mount the hard drive on startup TODO ensure sda1 will always be correct
sudo mkdir -p /mnt/idirfein_data
sudo chown pi:pi /mnt/idirfein_data
echo "/dev/sda1 /mnt/idirfein_data ext4 defaults 0 0" | sudo tee -a /etc/fstab

if [ {is_lan_only} ]; then
    # Reboot to apply changes
    touch "$MARKER_FILE"

    sudo reboot
fi

# Set up DuckDNS
mkdir -p ~/duckdns
cd ~/duckdns

cat <<EOL > duck.sh
#!/bin/bash
echo url="https://www.duckdns.org/update?domains={duckdns_domain}&token={duckdns_token}&ip=" | curl -k -o ~/duckdns/duck.log -K -
EOL

chmod 700 duck.sh

# Add the DuckDNS script to crontab
(crontab -l 2>/dev/null; echo "*/5 * * * * ~/duckdns/duck.sh >/dev/null 2>&1") | crontab -

# TODO set static ip

# Set up SSL certificates using Let's Encrypt
sudo apt install -y certbot
sudo certbot certonly --standalone -d {duckdns_domain}.duckdns.org --non-interactive --agree-tos --email {certbot_email}

# Reboot to apply changes
touch "$MARKER_FILE"

sudo reboot
        "#
    ).replace('\'', r"'\''");

    let idirfein_data_dir = dirs::data_dir()
        .expect("Can't find data dir")
        .as_path()
        .join(APP_ID);
    let sd_first_run_service_path = PathBuf::from(mnt_dir)
        .join("etc/systemd/system/first_boot.service")
        .to_string_lossy()
        .to_string();
    let sd_first_run_service_root_folder = PathBuf::from(mnt_dir)
        .join("etc/systemd/system")
        .to_string_lossy()
        .to_string();
    let sd_first_run_script_path = PathBuf::from(mnt_dir)
        .join("etc/first_boot.sh")
        .to_string_lossy()
        .to_string();
    let sd_idirfein_binary_path = PathBuf::from(mnt_dir)
        .join("home/pi/idirfein_server")
        .to_string_lossy()
        .to_string();
    let sd_idirfein_config_root_folder = PathBuf::from(mnt_dir)
        .join("home/pi/.config/idirfein_server")
        .to_string_lossy()
        .to_string();
    let sd_idirfein_config_path = PathBuf::from(mnt_dir)
        .join("home/pi/.config/idirfein_server/config.json")
        .to_string_lossy()
        .to_string();
    let idirfein_server_path = idirfein_data_dir
        .join("idirfein_server")
        .to_string_lossy()
        .to_string();

    let setup_commands = format!(
        r#"
mkdir -p {mnt_dir} &&
umount {root_fs_partition_name} &&
mount {root_fs_partition_name} {mnt_dir} &&
mkdir -p {sd_first_run_service_root_folder} &&
echo '{setup_service_content}' > {sd_first_run_service_path} &&
echo '{setup_script_content}' > {sd_first_run_script_path} &&
mkdir -p {sd_idirfein_config_root_folder} &&
echo '{serialised_config}' > {sd_idirfein_config_path} &&
cp {idirfein_server_path} {sd_idirfein_binary_path} &&
ln -s ../first_boot.service  {mnt_dir}/etc/systemd/system/multi-user.target.wants/first_boot.service &&
umount {mnt_dir}
        "#
    );

    let setup_command = tokio::process::Command::new("pkexec") // TODO put this back to flatpak
        .arg("sh")
        .arg("-c")
        .arg(setup_commands)
        .output();
    let finished = setup_command.await.map_err(|err| err.to_string())?;
    println!(
        "Setup command output stderr: {} stdout: {}",
        String::from_utf8_lossy(&finished.stderr),
        String::from_utf8_lossy(&finished.stdout)
    );

    // TODO
    // write server binary
    // write server password to /etc/shadow
    // set up duckdns if required
    // set static ip
    // TODO figure out ssl certs
    // Write server config file, this includes users list
    // put server bin on start up apps
    // put script which automounts harddrive in start up apps
    Ok(())
}
