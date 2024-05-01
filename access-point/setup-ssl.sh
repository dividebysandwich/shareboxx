#!/bin/bash
apt install -y nginx
openssl req -x509 -nodes -days 9000 -newkey rsa:2048 -keyout /etc/ssl/private/nginx-selfsigned.key -out /etc/ssl/certs/nginx-selfsigned.crt
cp nginx.conf /etc/nginx/sites-available/shareboxx
ln -s /etc/nginx/sites-available/shareboxx /etc/nginx/sites-enabled/
rm /etc/nginx/sites-enabled/default
systemctl restart nginx
