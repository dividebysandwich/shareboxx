# shareboxx
An anonymous, disconnected local filesharing system over WiFi, similar to Piratebox and Librarybox, entirely written in Rust.

![image](https://github.com/dividebysandwich/shareboxx/assets/23048489/72fe97a9-5345-4e76-8d25-debd9dac55bd)

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

- Get a small linux computer. See the [Wiki](https://github.com/dividebysandwich/shareboxx/wiki) for hardware suggestions
- Install rust (see https://rustup.rs for instructions)
- Clone repository on your small linux computer or raspberry pi: ```git clone https://github.com/dividebysandwich/shareboxx```
- Compile: ```cargo install cargo-leptos&&cargo leptos build --release```
- ```cd access-point```
- Edit hostapd.conf and dnsmasq.conf to taste - You might want to get an extra wifi adapter with a good antenna, see the [Wiki](https://github.com/dividebysandwich/shareboxx/wiki) for suggested dongles and additional instructions. If you decide to use an USB wifi adapter instead of the built-in Wifi, remember to change ```wlan0``` to ```wlan1``` in hostapd.conf and dnsmasq.conf
- Run ```sudo ./enable-captive-portal.sh```
- Run ```sudo ./setup-server.sh```
- Run ```sudo ./setup-ssl.sh```

Done! You should now be able to connect to the Shareboxx access point and be directed to the Shareboxx main page. You can use the web UI to copy files to /shareboxx/files, or you can copy files onto a USB drive and mount that under /shareboxx/files for example.
You may want to install a malware detection tool like [LMD](https://www.rfxn.com/projects/linux-malware-detect/) to automatically scan uploaded files.
