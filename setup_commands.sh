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

