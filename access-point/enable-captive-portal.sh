#!/bin/sh

# Install dhcpd
apt-get update
apt install -y nala 
DEBIAN_FRONTEND=noninteractive
nala install -y dhcpcd dnsmasq hostapd netfilter-persistent iptables-persistent

# Stop services since configuration files are not ready yet
systemctl stop dnsmasq
systemctl stop hostapd

# Raspberry Pi acts as router on wirless network
# As it runs a DHCP Server, the Raspi needs a static IP address
cat dhcpcd.conf | tee -a /etc/dhcpcd.conf > /dev/null
systemctl restart dhcpcd

# --- Configure DHCP server (dnsmasq)
if test -f /etc/dnsmasq.conf; then
    # Backup file
    mv /etc/dnsmasq.conf /etc/dnsmasq.conf.orig
fi
cp dnsmasq.conf /etc/dnsmasq.conf

# Don't let dnsmasq alter your /etc/resolv.conf file
# https://raspberrypi.stackexchange.com/questions/37439/proper-way-to-prevent-dnsmasq-from-overwriting-dns-server-list-supplied-by-dhcp
echo "DNSMASQ_EXCEPT=lo" | tee -a /etc/default/dnsmasq > /dev/null

systemctl unmask dnsmasq.service
systemctl enable dnsmasq.service
systemctl restart dnsmasq

# --- Routing and masquerade
# Activate IPv4 package forwarding
sed -i 's/#net.ipv4.ip_forward=1/net.ipv4.ip_forward=1/g' /etc/sysctl.conf
# Add redirect for all inbound http traffic for 192.168.4.1
iptables -t nat -I PREROUTING -p tcp --dport 80 -j DNAT --to-destination 192.168.4.1:3000
iptables -t nat -I PREROUTING -p tcp --dport 443 -j DNAT --to-destination 192.168.4.1:3443

# Comment out this line if you want to access the Pi via SSH when being connected
# to the Wifi Access Point. You can use: ssh -i "path/to/private/key/file" pi@192.168.4.1
# sudo iptables -t nat -I PREROUTING -p tcp --dport 22 -j ACCEPT

# Save to be loaded at boot by the netfilter-persistent service
netfilter-persistent save

# --- Configure access point (hostapd)
# Make sure wlan is not blocked on raspi
rfkill unblock wlan
cp hostapd.conf /etc/hostapd/hostapd.conf
systemctl unmask hostapd
systemctl enable hostapd
systemctl start hostapd
