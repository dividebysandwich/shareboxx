# shareboxx
An anonymous, disconnected local filesharing system over WiFi, similar to Piratebox and Librarybox, entirely written in Rust.

<img width="995" height="874" alt="image" src="https://github.com/user-attachments/assets/8105d438-3aa7-4fbe-9232-0eafa2e0eaaa" />

### What does it do?

This software turns your small linux computer or Raspberry Pi into an wireless anonymous offline local filesharing (WAOLF?) system. Anyone within WiFi range can connect to Shareboxx and freely download and upload files. You can install Shareboxx at a fixed location, or bring a Powerbank and take it with you. The idea is similar to the now-abandoned Piratebox, except Shareboxx does not have a large footprint of python scripts, html and javascript files. It's a single executable that does everything: Serving files, the web UI with chat, and accepting and processing uploads.

### Features:
- Quick and easy directory browsing
- Supports large file downloads and uploads
- Files can be uploaded to any directory
- Chat function with live updates
- File overwrite protection
- Responsive UI
- No accounts needed, minimal logging

### Hardware requirements:

You'll want a small PC or a single board computer of some sort, like a Raspberry Pi. Shareboxx does not require a lot of memory or CPU power, but you might want to invest in enough flash storage for things like movies. Finally, a good wifi adapter with a large antenna, or one with SMA connector and a good external antenna is recommended for maximum range.

### What not to do:

Do not put Shareboxx on the internet. It is meant to be run on an isolated system with local wifi being the only means of connecting.

### Installation:

#### Option A — Debian / Raspberry Pi OS (.deb package, recommended)

- Get a small linux computer. See the [Wiki](https://github.com/dividebysandwich/shareboxx/wiki) for hardware suggestions
- Download the latest `shareboxx_*.deb` for your architecture from the [Releases page](https://github.com/dividebysandwich/shareboxx/releases)
- Install: `sudo apt install ./shareboxx_*.deb`
- The post-install step will detect your wireless adapter(s), let you pick one, and configure the access point, captive portal, and SSL automatically
- To re-run setup later: `sudo shareboxx-setup`

#### Option B — Build from source (any distro)

- Install rust (see https://rustup.rs for instructions)
- Install the runtime dependencies through your distro's package manager: `hostapd dnsmasq nginx openssl iw iproute2 iptables` (on Debian-based systems also `netfilter-persistent iptables-persistent`)
- Clone the repository: `git clone https://github.com/dividebysandwich/shareboxx`
- Compile: `cargo install cargo-leptos && cargo leptos build --release`
- Run the all-in-one installer: `sudo ./access-point/install-from-source.sh`
- It will detect your wireless adapter(s), let you select which one to use, and configure everything (binary, systemd service, dnsmasq, hostapd, iptables, nginx + self-signed SSL)

Done! You should now be able to connect to the Shareboxx access point and be directed to the Shareboxx main page. The web UI lets you upload to `/var/lib/shareboxx/files` directly. For larger libraries, see "Using a USB stick for storage" below.

You may want to install a [malware detection tool](https://github.com/dividebysandwich/shareboxx/wiki/How-to-set-up-a-malware-scanner-to-automatically-scan-uploads) to automatically scan uploaded files.

### Using a USB stick for storage

Shareboxx serves whatever lives in `/var/lib/shareboxx/files/`. To back the share with a USB stick (or any external disk), mount it there. Two recipes:

**1) Mount the USB stick directly at the share path** — simplest if the stick is dedicated to Shareboxx.

```bash
sudo systemctl stop shareboxx
lsblk                                  # find your device, e.g. /dev/sda1
sudo mkdir -p /var/lib/shareboxx/files

# /etc/fstab entry — pick the right filesystem type:
# exFAT/FAT32 stick (no Unix permissions, so uid/gid are required):
echo '/dev/sda1  /var/lib/shareboxx/files  exfat  defaults,nofail,uid=shareboxx,gid=shareboxx  0 2' \
     | sudo tee -a /etc/fstab
# ext4/btrfs/xfs stick (drop uid/gid; chown once after mount):
# echo '/dev/sda1  /var/lib/shareboxx/files  auto   defaults,nofail  0 2' | sudo tee -a /etc/fstab

sudo mount /var/lib/shareboxx/files
sudo chown -R shareboxx:shareboxx /var/lib/shareboxx/files   # ext4/btrfs/xfs only
sudo systemctl start shareboxx
```

**2) Bind-mount an already-mounted path** — useful if the stick is already mounted elsewhere (e.g. `/mnt/bigdisk`), or if you want to share an existing folder without moving it.

```bash
sudo systemctl stop shareboxx
sudo rsync -a /var/lib/shareboxx/files/ /mnt/bigdisk/shareboxx-files/
sudo chown -R shareboxx:shareboxx /mnt/bigdisk/shareboxx-files
echo '/mnt/bigdisk/shareboxx-files  /var/lib/shareboxx/files  none  bind  0 0' \
     | sudo tee -a /etc/fstab
sudo mount /var/lib/shareboxx/files
sudo systemctl start shareboxx
```

Notes:
- `nofail` lets the Pi boot even if the stick isn't plugged in; Shareboxx will then serve an empty directory until you reconnect it.
- For a stick that's plugged/unplugged at runtime, replace the fstab entry with a systemd `.mount` + `.automount` pair so Shareboxx auto-recovers.
- If the disk is on its own mount point, ensure it comes up before `shareboxx.service` starts — the simplest way is to add `x-systemd.before=shareboxx.service` to the fstab options.

### Development

To run locally for development/testing, execute ```cargo leptos watch```
