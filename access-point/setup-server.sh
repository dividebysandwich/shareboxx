#!/bin/bash
useradd shareboxx
mkdir /shareboxx
mkdir /shareboxx/files
touch /shareboxx/chat.json
chmod +w /shareboxx/chat.json
chown shareboxx /shareboxx -R
cp -r ../target/site /shareboxx/
cp ../target/release/shareboxx /usr/bin/
cp ./shareboxx.service /etc/systemd/system/
systemctl enable shareboxx
systemctl start shareboxx
