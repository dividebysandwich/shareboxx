#!/bin/bash
useradd shareboxx
mkdir /shareboxx
mkdir /shareboxx/files
chown shareboxx /shareboxx -R
cp -r ../target/site /shareboxx/
cp ../target/release/shareboxx /usr/bin/
cp ./shareboxx.service /etc/systemd/system/
systemctl enable shareboxx
systemctl start shareboxx
