# shareboxx
An anonymous, disconnected local filesharing system over WiFi, similar to Piratebox and Librarybox, entirely written in Rust.

![image](https://github.com/dividebysandwich/shareboxx/assets/23048489/10144b44-464c-4fc1-a3d5-bbd423c66048)

### What does it do?

This software can turn your small linux computer or Raspberry Pi into an wireless anonymous offline local filesharing (WAOLF?) system. The idea is similar to the now-abandoned Piratebox, except Shareboxx does not have a large footprint of webservers and php script files. It's a single executable that does everything: Serving files, the web UI, and accepting uploads.

### Hardware requirements:

You'll want a small PC or a single board computer of some sort, like a Raspberry Pi. Shareboxx does not require a lot of memory or CPU power, but you might want to invest in enough flash storage for things like movies. Finally, a good wifi adapter with a large antenna, or one with SMA connector and a good external antenna is recommended for maximum range.

### What not to do:

Do not put Shareboxx on the internet. It is meant to be run on an isolated system with local wifi being the only means of connecting.

### Installation:

- Install rust (see https://rustup.rs for instructions)
- Clone repository on your small linux computer or raspberry pi: ```git clone https://github.com/dividebysandwich/shareboxx```
- Compile: ```cargo install cargo-leptos&&cargo leptos build --release```
- ```cd access-point```
- Edit hostapd.conf and dnsmasq.conf to taste
- Run ```sudo ./enable-captive-portal.sh```
- Run ```sudo ./setup-server.sh```

Done! You should now be able to connect to the Shareboxx access point and be directed to the Shareboxx main page. You can use the web UI to copy files to /shareboxx/files, or you can copy files onto a USB drive and mount that under /shareboxx/files for example.
