#!/bin/bash
mkdir /shareboxx
mkdir /shareboxx/files
cp -r ../target/site /shareboxx/
cp ../target/release/shareboxx /usr/local/bin/
cp ./shareboxx.service /etc/systemd/system/
systemctl enable shareboxx
systemctl start shareboxx
