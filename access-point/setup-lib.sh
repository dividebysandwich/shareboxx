# shellcheck shell=bash
#
# ShareBoxx access-point setup library.
#
# Sourced by shareboxx-setup (installed at /usr/bin/shareboxx-setup) and
# install-from-source.sh. Do not execute directly.
#
# Shipped by the .deb at /usr/lib/shareboxx/setup-lib.sh; lives next to the
# entry-point scripts in the source tree.
#
# Conventions:
#   - This file does NOT set `set -e`; it inherits the parent's options.
#   - Functions write progress with info/ok/warn/err/step.
#   - Output variables are globals (no `local`) by design — both entry-point
#     scripts read them after calling the corresponding functions.

# ── Colours / log helpers ───────────────────────────────────────────────────

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}[info]${NC}  $*"; }
ok()    { echo -e "${GREEN}[ok]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC}  $*"; }
err()   { echo -e "${RED}[error]${NC} $*" >&2; }
step()  { echo -e "\n${BOLD}── $* ──${NC}"; }

ask() {
    # The prompt MUST go to stderr — `ask` is invoked via command substitution
    # (`VAR=$(ask ...)`), which captures stdout. If the prompt were on stdout
    # it would end up inside VAR instead of on the user's terminal.
    local prompt="$1" default="$2" reply
    echo -en "${BOLD}$prompt${NC} [${default}]: " >&2
    read -r reply
    echo "${reply:-$default}"
}

confirm() {
    local prompt="$1" default="${2:-n}" reply
    local hint="[y/N]"; [[ "$default" == "y" ]] && hint="[Y/n]"
    echo -en "${BOLD}$prompt${NC} $hint: "
    read -r reply
    reply="${reply:-$default}"
    [[ "$reply" =~ ^[Yy] ]]
}

# ── Distro detection ────────────────────────────────────────────────────────

DISTRO_ID=""; DISTRO_LIKE=""
if [[ -r /etc/os-release ]]; then
    # shellcheck disable=SC1091
    . /etc/os-release || true
    DISTRO_ID="${ID:-}"
    DISTRO_LIKE="${ID_LIKE:-}"
fi

is_fedora_like() {
    case "$DISTRO_ID"   in fedora|rhel|centos|rocky|almalinux) return 0 ;; esac
    case "$DISTRO_LIKE" in *fedora*|*rhel*) return 0 ;; esac
    return 1
}

# ── Shared paths / markers ──────────────────────────────────────────────────

MARKER="# --- ShareBoxx AP config ---"
DHCPCD_CONF="/etc/dhcpcd.conf"
DNSMASQ_CONF="/etc/dnsmasq.d/shareboxx.conf"
NM_UNMANAGE_CONF="/etc/NetworkManager/conf.d/shareboxx-unmanaged.conf"
HOSTAPD_CONF="/etc/hostapd/hostapd.conf"
CERT_PATH="/etc/ssl/certs/shareboxx-selfsigned.crt"
KEY_PATH="/etc/ssl/private/shareboxx-selfsigned.key"
AP_UNIT="/etc/systemd/system/shareboxx-ap.service"
DNSMASQ_DROPIN="/etc/systemd/system/dnsmasq.service.d/shareboxx.conf"
HOSTAPD_DROPIN="/etc/systemd/system/hostapd.service.d/shareboxx.conf"

# ── iptables persistence (distro-aware) ─────────────────────────────────────

persist_iptables() {
    if command -v netfilter-persistent &>/dev/null; then
        netfilter-persistent save 2>/dev/null || true
    elif is_fedora_like; then
        mkdir -p /etc/sysconfig
        iptables-save > /etc/sysconfig/iptables 2>/dev/null || true
        systemctl enable iptables 2>/dev/null || true
    elif command -v iptables-save &>/dev/null; then
        # Arch / generic
        mkdir -p /etc/iptables
        iptables-save > /etc/iptables/iptables.rules 2>/dev/null || true
        systemctl enable iptables 2>/dev/null || true
    fi
}

remove_iptables_rules() {
    # Delete every rule whose comment starts with "shareboxx-".
    #
    # NOTE: the trailing `|| true` is essential under `set -euo pipefail`.
    # When grep finds no matches it exits 1, and pipefail then makes the
    # whole pipeline fail, which (combined with set -e) would silently
    # abort the script. That bit us once already — don't remove it.
    local table rule
    for table in nat filter; do
        while :; do
            rule=$(iptables -t "$table" -S 2>/dev/null | grep -F 'shareboxx-' | head -1) || true
            [[ -z "$rule" ]] && break
            # shellcheck disable=SC2086
            iptables -t "$table" $(echo "$rule" | sed 's/^-A/-D/') 2>/dev/null || break
        done
    done
}

# ── Uninstall path ──────────────────────────────────────────────────────────

do_uninstall() {
    step "Reverting ShareBoxx access-point configuration"

    info "Stopping services"
    systemctl stop shareboxx hostapd dnsmasq shareboxx-ap 2>/dev/null || true
    systemctl disable hostapd dnsmasq shareboxx-ap 2>/dev/null || true

    # Remove drop-ins BEFORE the AP unit so dnsmasq/hostapd don't end up
    # referencing a missing Requires= target between operations.
    if [[ -f "$DNSMASQ_DROPIN" || -f "$HOSTAPD_DROPIN" ]]; then
        rm -f "$DNSMASQ_DROPIN" "$HOSTAPD_DROPIN"
        rmdir --ignore-fail-on-non-empty /etc/systemd/system/dnsmasq.service.d 2>/dev/null || true
        rmdir --ignore-fail-on-non-empty /etc/systemd/system/hostapd.service.d 2>/dev/null || true
        ok "Removed dnsmasq/hostapd drop-ins"
    fi
    if [[ -f "$AP_UNIT" ]]; then
        rm -f "$AP_UNIT"
        ok "Removed $AP_UNIT"
    fi
    systemctl daemon-reload

    if [[ -f "$DHCPCD_CONF" ]] && grep -qF "$MARKER" "$DHCPCD_CONF"; then
        sed -i "/$MARKER/,/^$/d" "$DHCPCD_CONF"
        ok "Removed ShareBoxx block from $DHCPCD_CONF"
        systemctl restart dhcpcd 2>/dev/null || true
    fi

    if [[ -f "$DNSMASQ_CONF" ]]; then
        rm -f "$DNSMASQ_CONF"
        ok "Removed $DNSMASQ_CONF"
    fi
    if [[ -f /etc/dnsmasq.conf.shareboxx-backup ]]; then
        mv /etc/dnsmasq.conf.shareboxx-backup /etc/dnsmasq.conf
        ok "Restored /etc/dnsmasq.conf from backup"
    fi

    if [[ -f "$HOSTAPD_CONF" ]] && grep -qF "ShareBoxx" "$HOSTAPD_CONF"; then
        rm -f "$HOSTAPD_CONF"
        ok "Removed $HOSTAPD_CONF"
    fi

    if [[ -f "$NM_UNMANAGE_CONF" ]]; then
        rm -f "$NM_UNMANAGE_CONF"
        systemctl reload NetworkManager 2>/dev/null || true
        ok "Removed NetworkManager unmanage rule"
    fi

    remove_iptables_rules
    persist_iptables
    ok "Removed ShareBoxx iptables rules"

    rm -f /etc/nginx/sites-available/shareboxx \
          /etc/nginx/sites-enabled/shareboxx \
          /etc/nginx/conf.d/shareboxx.conf
    if command -v nginx &>/dev/null && nginx -t &>/dev/null; then
        systemctl reload nginx 2>/dev/null || true
    fi
    ok "Removed nginx ShareBoxx site"

    echo ""
    info "Access-point configuration reverted."
}

# ── IPv4 helpers ────────────────────────────────────────────────────────────

is_ipv4() {
    [[ "$1" =~ ^([0-9]{1,3}\.){3}[0-9]{1,3}$ ]] || return 1
    local IFS=.; local -a o=($1)
    local n
    for n in "${o[@]}"; do (( n >= 0 && n <= 255 )) || return 1; done
}

ip_to_int() {
    local IFS=.; local -a o=($1)
    echo $(( (o[0]<<24) + (o[1]<<16) + (o[2]<<8) + o[3] ))
}

# ── WiFi adapter detection / selection (sets IFACE) ─────────────────────────

describe_iface() {
    local ifc="$1" details="" mac driver mode
    mac=$(cat "/sys/class/net/$ifc/address" 2>/dev/null || echo "")
    driver=$(basename "$(readlink -f "/sys/class/net/$ifc/device/driver" 2>/dev/null)" 2>/dev/null || echo "")
    if command -v iw &>/dev/null; then
        mode=$(iw dev "$ifc" info 2>/dev/null | awk '/type/{print $2; exit}')
    fi
    [[ -n "$mac"    ]] && details+="mac=$mac "
    [[ -n "$driver" ]] && details+="driver=$driver "
    [[ -n "$mode"   ]] && details+="mode=$mode"
    echo "$details"
}

select_iface() {
    step "WiFi Interface Detection"

    WIFI_IFACES=()
    if command -v iw &>/dev/null; then
        mapfile -t WIFI_IFACES < <(iw dev 2>/dev/null | awk '/Interface/{print $2}')
    fi
    if [[ ${#WIFI_IFACES[@]} -eq 0 ]]; then
        local w
        for w in /sys/class/net/*/wireless; do
            [[ -d "$w" ]] && WIFI_IFACES+=("$(basename "$(dirname "$w")")")
        done
    fi

    if [[ ${#WIFI_IFACES[@]} -eq 0 ]]; then
        warn "No WiFi interfaces detected."
        warn "Make sure your WiFi adapter is plugged in and drivers are loaded."
        IFACE=$(ask "WiFi interface (manual entry)" "wlan0")
        return
    fi
    if [[ ${#WIFI_IFACES[@]} -eq 1 ]]; then
        IFACE="${WIFI_IFACES[0]}"
        info "Found one WiFi interface: $IFACE  ($(describe_iface "$IFACE"))"
        return
    fi
    echo "Available WiFi interfaces:"
    local i
    for i in "${!WIFI_IFACES[@]}"; do
        printf "  [%d] %-10s %s\n" "$((i+1))" "${WIFI_IFACES[i]}" "$(describe_iface "${WIFI_IFACES[i]}")"
    done
    echo ""
    local sel
    while true; do
        echo -en "${BOLD}Select WiFi interface${NC} [1-${#WIFI_IFACES[@]}, default 1]: "
        read -r sel
        sel="${sel:-1}"
        if [[ "$sel" =~ ^[0-9]+$ ]] && (( sel >= 1 && sel <= ${#WIFI_IFACES[@]} )); then
            IFACE="${WIFI_IFACES[$((sel-1))]}"
            return
        fi
        echo "  Invalid selection. Enter 1-${#WIFI_IFACES[@]}."
    done
}

# ── AP-mode capability check ────────────────────────────────────────────────

check_ap_capability() {
    local ifc="$1"
    if ! command -v iw &>/dev/null; then
        warn "iw not available — cannot verify AP-mode capability."
        return
    fi
    local phy
    phy=$(iw dev "$ifc" info 2>/dev/null | awk '/wiphy/{print $2; exit}')
    if [[ -z "$phy" ]]; then
        warn "Could not determine wiphy for $ifc; skipping AP-mode check."
        return
    fi
    if iw phy "phy$phy" info 2>/dev/null | grep -qE '^[[:space:]]+\* AP$'; then
        ok "$ifc supports AP mode"
    else
        err "$ifc does NOT advertise AP mode."
        err "This adapter likely cannot host a WiFi network."
        confirm "Continue anyway?" n || exit 1
    fi
}

# ── Service-conflict detection (sets SYSTEMD_RESOLVED_ACTIVE) ───────────────

SYSTEMD_RESOLVED_ACTIVE=0

check_service_conflicts() {
    local ifc="$1"

    # NetworkManager: tell it to leave the AP interface alone.
    if systemctl is-active --quiet NetworkManager 2>/dev/null \
       && command -v nmcli &>/dev/null; then
        local nm_state
        nm_state=$(nmcli -t -f DEVICE,STATE device 2>/dev/null \
                    | awk -F: -v d="$ifc" '$1==d{print $2; exit}')
        if [[ -n "$nm_state" && "$nm_state" != "unmanaged" ]]; then
            warn "NetworkManager is managing $ifc (state: $nm_state)."
            warn "It will fight dhcpcd/hostapd for the interface."
            if confirm "Tell NetworkManager to leave $ifc alone?" y; then
                mkdir -p "$(dirname "$NM_UNMANAGE_CONF")"
                cat > "$NM_UNMANAGE_CONF" <<EOF
# Installed by shareboxx-setup; do not edit manually.
[keyfile]
unmanaged-devices=interface-name:$ifc
EOF
                systemctl reload NetworkManager 2>/dev/null \
                    || systemctl restart NetworkManager 2>/dev/null || true
                nmcli device set "$ifc" managed no 2>/dev/null || true
                ok "NetworkManager will ignore $ifc"
            fi
        fi
    fi

    # wpa_supplicant: stop it on the AP interface.
    if pgrep -af "wpa_supplicant.*-i[= ]?$ifc" >/dev/null 2>&1 \
       || pgrep -af "wpa_supplicant.*$ifc" >/dev/null 2>&1; then
        warn "wpa_supplicant is running on $ifc; it will conflict with hostapd."
        if confirm "Stop wpa_supplicant on $ifc?" y; then
            systemctl stop "wpa_supplicant@$ifc" 2>/dev/null || true
            systemctl disable "wpa_supplicant@$ifc" 2>/dev/null || true
            pkill -f "wpa_supplicant.*$ifc" 2>/dev/null || true
            ok "wpa_supplicant stopped on $ifc"
        fi
    fi

    # systemd-resolved holds 127.0.0.53:53. We don't disable it; instead we
    # tell dnsmasq to bind only to $IFACE so they coexist.
    if systemctl is-active --quiet systemd-resolved 2>/dev/null; then
        info "systemd-resolved is active; dnsmasq will be scoped to $ifc only."
        SYSTEMD_RESOLVED_ACTIVE=1
    fi
}

# ── Validated configuration prompts ─────────────────────────────────────────
# Sets: SSID, CHANNEL, COUNTRY, AP_IP, NETWORK_PREFIX, AP_INT,
#       DHCP_START, DHCP_END, SUBNET

prompt_config() {
    step "Configuration"
    echo -e "Configure your ShareBoxx access point. Press Enter to accept defaults.\n"
    info "WiFi interface: $IFACE"

    # SSID: 1-32 bytes
    while true; do
        SSID=$(ask "Access point name (SSID)" "ShareBoxx")
        [[ -z "$SSID" ]] && { echo "  SSID cannot be empty."; continue; }
        (( $(printf '%s' "$SSID" | wc -c) > 32 )) && { echo "  SSID must be ≤32 bytes."; continue; }
        break
    done

    # Channel: 1-14, warn for 12-14 (non-US/CA)
    while true; do
        CHANNEL=$(ask "WiFi channel (1-11 for US/CA, 1-13 EU, 1-14 JP)" "6")
        if [[ ! "$CHANNEL" =~ ^[0-9]+$ ]] || (( CHANNEL < 1 || CHANNEL > 14 )); then
            echo "  Channel must be 1-14."; continue
        fi
        if (( CHANNEL > 11 )); then
            warn "Channel $CHANNEL is restricted in many countries (US/CA: 1-11)."
            confirm "Use channel $CHANNEL anyway?" n || continue
        fi
        break
    done

    # Country: 2 letters
    while true; do
        COUNTRY=$(ask "Country code (ISO 3166, 2 letters)" "US")
        COUNTRY=$(echo "$COUNTRY" | tr '[:lower:]' '[:upper:]')
        [[ "$COUNTRY" =~ ^[A-Z]{2}$ ]] && break
        echo "  Must be a 2-letter code (e.g. US, DE, JP)."
    done

    # AP IP
    while true; do
        AP_IP=$(ask "Access point IP address (/24 assumed)" "192.168.4.1")
        is_ipv4 "$AP_IP" && break
        echo "  Not a valid IPv4 address."
    done
    NETWORK_PREFIX="${AP_IP%.*}"
    AP_INT=$(ip_to_int "$AP_IP")

    # DHCP range
    while true; do
        DHCP_START=$(ask "DHCP range start" "${NETWORK_PREFIX}.2")
        if ! is_ipv4 "$DHCP_START" || [[ "${DHCP_START%.*}" != "$NETWORK_PREFIX" ]]; then
            echo "  Must be a valid IPv4 in $NETWORK_PREFIX.0/24."; continue
        fi
        [[ "$DHCP_START" == "$AP_IP" ]] && { echo "  Cannot be the AP IP."; continue; }
        break
    done
    while true; do
        DHCP_END=$(ask "DHCP range end" "${NETWORK_PREFIX}.254")
        if ! is_ipv4 "$DHCP_END" || [[ "${DHCP_END%.*}" != "$NETWORK_PREFIX" ]]; then
            echo "  Must be a valid IPv4 in $NETWORK_PREFIX.0/24."; continue
        fi
        if (( $(ip_to_int "$DHCP_END") < $(ip_to_int "$DHCP_START") )); then
            echo "  End must be ≥ start."; continue
        fi
        if (( AP_INT >= $(ip_to_int "$DHCP_START") && AP_INT <= $(ip_to_int "$DHCP_END") )); then
            warn "AP IP $AP_IP is inside the DHCP range; it will be excluded automatically."
        fi
        break
    done
    SUBNET="255.255.255.0"

    echo ""
    info "Interface:  $IFACE"
    info "SSID:       $SSID  (open network — ShareBoxx is passwordless by design)"
    info "Channel:    $CHANNEL"
    info "Country:    $COUNTRY"
    info "AP IP:      $AP_IP/24"
    info "DHCP range: $DHCP_START – $DHCP_END"
    echo ""
    confirm "Proceed with setup?" y || { info "Aborted."; exit 0; }
}

# ── Service configuration ───────────────────────────────────────────────────

configure_ap_interface() {
    step "Configuring AP interface (synchronous static IP via systemd unit)"

    # Stop any previous run of the unit so its old ExecStop removes the
    # previously assigned address before we overwrite the unit file.
    if [[ -f "$AP_UNIT" ]]; then
        systemctl stop shareboxx-ap.service 2>/dev/null || true
    fi

    # If dhcpcd is in the picture, tell it to ignore the AP interface — we
    # manage the IP ourselves below. Also strip any legacy ShareBoxx block
    # from older installs that configured a static IP via dhcpcd.
    if [[ -f "$DHCPCD_CONF" ]]; then
        if grep -qF "$MARKER" "$DHCPCD_CONF"; then
            sed -i "/$MARKER/,/^$/d" "$DHCPCD_CONF"
        fi
        cat >> "$DHCPCD_CONF" <<EOF
$MARKER
denyinterfaces $IFACE

EOF
        systemctl restart dhcpcd 2>/dev/null || true
    fi

    local ip_bin
    ip_bin=$(command -v ip || true)
    if [[ -z "$ip_bin" ]]; then
        err "'ip' command not found — install iproute2."
        exit 1
    fi

    # The unit applies the IP synchronously before hostapd/dnsmasq start.
    # Type=oneshot + RemainAfterExit=yes lets dependents wait for it cleanly.
    cat > "$AP_UNIT" <<EOF
[Unit]
Description=ShareBoxx access-point interface setup ($IFACE)
After=sys-subsystem-net-devices-${IFACE}.device
Wants=sys-subsystem-net-devices-${IFACE}.device
Before=hostapd.service dnsmasq.service

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart=$ip_bin link set $IFACE up
ExecStart=$ip_bin addr replace ${AP_IP}/24 dev $IFACE
ExecStop=-$ip_bin addr del ${AP_IP}/24 dev $IFACE

[Install]
WantedBy=multi-user.target
EOF

    # Drop-ins so hostapd/dnsmasq won't start until the IP is on the
    # interface — this is what fixes the dnsmasq "Cannot assign requested
    # address" race seen with the dhcpcd-based static-IP approach.
    mkdir -p "$(dirname "$DNSMASQ_DROPIN")" "$(dirname "$HOSTAPD_DROPIN")"
    cat > "$DNSMASQ_DROPIN" <<EOF
[Unit]
After=shareboxx-ap.service
Requires=shareboxx-ap.service
EOF
    cat > "$HOSTAPD_DROPIN" <<EOF
[Unit]
After=shareboxx-ap.service
Requires=shareboxx-ap.service
EOF

    systemctl daemon-reload
    systemctl enable shareboxx-ap.service
    ok "shareboxx-ap.service installed (sets ${AP_IP}/24 on $IFACE before dnsmasq/hostapd)"
}

configure_dnsmasq() {
    step "Configuring DHCP server (dnsmasq)"

    mkdir -p /etc/dnsmasq.d
    if [[ -f /etc/dnsmasq.conf && ! -f /etc/dnsmasq.conf.shareboxx-backup ]]; then
        cp /etc/dnsmasq.conf /etc/dnsmasq.conf.shareboxx-backup
        info "Backed up /etc/dnsmasq.conf"
    fi
    if [[ -f /etc/dnsmasq.conf ]] && ! grep -qE '^\s*conf-dir=/etc/dnsmasq\.d' /etc/dnsmasq.conf; then
        echo "conf-dir=/etc/dnsmasq.d/,*.conf" >> /etc/dnsmasq.conf
    fi

    {
        echo "# ShareBoxx DHCP & captive-portal DNS"
        echo "interface=$IFACE"
        # bind-interfaces lets dnsmasq coexist with systemd-resolved on :53.
        echo "bind-interfaces"
        echo "except-interface=lo"
        echo "listen-address=$AP_IP"
        echo "dhcp-range=${DHCP_START},${DHCP_END},${SUBNET},24h"
        echo ""
        echo "# Redirect ALL domains to the access point (captive portal)"
        echo "address=/#/${AP_IP}"
    } > "$DNSMASQ_CONF"

    if [[ -f /etc/default/dnsmasq ]] && ! grep -qF "DNSMASQ_EXCEPT=lo" /etc/default/dnsmasq; then
        echo "DNSMASQ_EXCEPT=lo" >> /etc/default/dnsmasq
    fi
    systemctl unmask dnsmasq.service 2>/dev/null || true
    systemctl enable dnsmasq.service
    ok "dnsmasq configured (DHCP ${DHCP_START}–${DHCP_END}, captive DNS, bound to $IFACE)"
}

configure_hostapd() {
    step "Configuring WiFi access point (hostapd)"

    mkdir -p /etc/hostapd
    cat > "$HOSTAPD_CONF" <<EOF
# ShareBoxx WiFi Access Point (open network — ShareBoxx is passwordless)
interface=$IFACE
driver=nl80211
hw_mode=g
channel=$CHANNEL
ieee80211d=1
country_code=$COUNTRY
ieee80211n=1
wmm_enabled=1

ssid=$SSID
auth_algs=1
ap_isolate=1
EOF

    if [[ -f /etc/default/hostapd ]]; then
        sed -i 's|^#\?DAEMON_CONF=.*|DAEMON_CONF="/etc/hostapd/hostapd.conf"|' /etc/default/hostapd
    fi
    command -v rfkill &>/dev/null && rfkill unblock wlan 2>/dev/null || true
    systemctl unmask hostapd 2>/dev/null || true
    systemctl enable hostapd
    ok "hostapd configured (SSID: $SSID, channel $CHANNEL, open)"
}

configure_iptables_redirect() {
    step "Configuring firewall rules (captive portal)"

    if [[ -f /etc/sysctl.conf ]]; then
        sed -i 's/^#\?net.ipv4.ip_forward=.*/net.ipv4.ip_forward=1/' /etc/sysctl.conf
    fi
    sysctl -w net.ipv4.ip_forward=1 >/dev/null

    remove_iptables_rules

    iptables -t nat -I PREROUTING -i "$IFACE" -p tcp --dport 80 \
        -m comment --comment "shareboxx-http" \
        -j DNAT --to-destination "${AP_IP}:3000"
    iptables -t nat -I PREROUTING -i "$IFACE" -p tcp --dport 443 \
        -m comment --comment "shareboxx-https" \
        -j DNAT --to-destination "${AP_IP}:3443"
    iptables -A INPUT -i "$IFACE" -p tcp --dport 22 \
        -m comment --comment "shareboxx-ssh-block" -j DROP

    persist_iptables
    ok "iptables rules configured (HTTP→:3000, HTTPS→:3443, SSH blocked on $IFACE)"
}

# Sets: NGINX_SITE_CONF
configure_nginx_ssl() {
    step "Setting up SSL (nginx reverse proxy)"

    mkdir -p /etc/ssl/certs /etc/ssl/private
    local regen
    if [[ -f "$CERT_PATH" && -f "$KEY_PATH" ]]; then
        info "SSL certificate already exists, keeping it."
        confirm "Regenerate?" n && regen=1 || regen=0
    else
        regen=1
    fi
    if [[ "$regen" -eq 1 ]]; then
        openssl req -x509 -nodes -days 3650 \
            -newkey rsa:2048 \
            -keyout "$KEY_PATH" \
            -out "$CERT_PATH" \
            -subj "/CN=shareboxx.lan/O=ShareBoxx/C=$COUNTRY" \
            2>/dev/null
        ok "Self-signed certificate generated (valid 10 years)"
    fi

    # Pick the canonical location for this distro's nginx layout.
    if [[ -d /etc/nginx/sites-available && -d /etc/nginx/sites-enabled ]]; then
        NGINX_SITE_CONF="/etc/nginx/sites-available/shareboxx"
    elif [[ -d /etc/nginx/conf.d ]]; then
        NGINX_SITE_CONF="/etc/nginx/conf.d/shareboxx.conf"
    else
        mkdir -p /etc/nginx/conf.d
        NGINX_SITE_CONF="/etc/nginx/conf.d/shareboxx.conf"
    fi

    # Defensively remove ANY shareboxx site we did not pick — older installs
    # may have written to the other location, leaving a stale duplicate that
    # silently shadows the new config (or causes a duplicate default_server).
    local stale
    for stale in /etc/nginx/sites-available/shareboxx \
                 /etc/nginx/sites-enabled/shareboxx \
                 /etc/nginx/conf.d/shareboxx.conf; do
        if [[ "$stale" != "$NGINX_SITE_CONF" ]] \
           && [[ "$stale" != "/etc/nginx/sites-enabled/shareboxx" \
                 || "$NGINX_SITE_CONF" != /etc/nginx/sites-available/* ]] \
           && [[ -e "$stale" || -L "$stale" ]]; then
            rm -f "$stale"
            info "Removed stale $stale"
        fi
    done

    # If something has flagged the file immutable (chattr +i), the cat below
    # would silently fail and we'd end up with the old contents. Strip it.
    if command -v lsattr &>/dev/null && [[ -f "$NGINX_SITE_CONF" ]] \
       && lsattr "$NGINX_SITE_CONF" 2>/dev/null | awk '{print $1}' | grep -q i; then
        chattr -i "$NGINX_SITE_CONF" 2>/dev/null || true
        warn "Cleared immutable flag on $NGINX_SITE_CONF"
    fi

    cat > "$NGINX_SITE_CONF" <<EOF
# Generated by shareboxx-setup. Do not edit by hand — re-run shareboxx-setup
# to regenerate, or your changes will be lost.
server {
    listen 3443 ssl http2 default_server;
    listen [::]:3443 ssl http2;
    server_name shareboxx.lan $AP_IP;

    ssl_certificate     $CERT_PATH;
    ssl_certificate_key $KEY_PATH;

    # Large-upload friendliness — ShareBoxx is meant for sharing big files.
    client_max_body_size 10G;
    client_body_timeout 1h;
    send_timeout 1h;

    location / {
        proxy_pass         http://127.0.0.1:3000;
        proxy_redirect     off;
        proxy_set_header   Host \$host;
        proxy_set_header   X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header   X-Forwarded-Proto \$scheme;
        proxy_http_version 1.1;
        proxy_set_header   Upgrade \$http_upgrade;
        proxy_set_header   Connection "upgrade";

        # Stream the request body straight to the backend instead of
        # buffering it to a temp file first. Avoids "the upload finished
        # but the UI still says 50%" when nginx is slowly forwarding.
        proxy_request_buffering off;
        proxy_buffering         off;

        # Big-upload timeouts for slow Wi-Fi clients.
        proxy_read_timeout    1h;
        proxy_send_timeout    1h;
        proxy_connect_timeout 60s;
    }
}
EOF

    # Verify the directive actually made it onto disk before continuing —
    # catches weird filesystem / permission / write-protect cases.
    if ! grep -q "client_max_body_size 10G" "$NGINX_SITE_CONF"; then
        err "Wrote $NGINX_SITE_CONF but it does not contain the expected"
        err "'client_max_body_size 10G' directive. Refusing to continue."
        exit 1
    fi

    if [[ "$NGINX_SITE_CONF" == *sites-available* ]]; then
        ln -sf "$NGINX_SITE_CONF" /etc/nginx/sites-enabled/shareboxx
        rm -f /etc/nginx/sites-enabled/default
    fi

    if nginx -t 2>/dev/null; then
        ok "nginx configuration valid"
    else
        err "nginx configuration test failed!"
        nginx -t
        exit 1
    fi
    systemctl enable nginx
    ok "nginx configured at $NGINX_SITE_CONF (SSL reverse proxy on :3443, large-upload friendly)"
}

# Sets: ALL_OK
start_services_and_check() {
    step "Starting services"

    # shareboxx-ap.service must come up first — its drop-ins guarantee
    # hostapd/dnsmasq won't start until it has applied the IP.
    systemctl restart shareboxx-ap.service
    systemctl restart dnsmasq
    systemctl restart hostapd
    systemctl restart nginx
    systemctl restart shareboxx

    sleep 2
    ALL_OK=1
    local svc
    for svc in shareboxx-ap hostapd dnsmasq nginx shareboxx; do
        if systemctl is-active --quiet "$svc"; then
            ok "$svc is running"
        else
            err "$svc failed to start"
            ALL_OK=0
        fi
    done
}

# Reads: ALL_OK, SSID, AP_IP, NGINX_SITE_CONF, DHCPCD_CONF, DNSMASQ_CONF, HOSTAPD_CONF
print_summary() {
    step "Setup Complete"

    if [[ ${ALL_OK:-0} -eq 1 ]]; then
        echo -e "${GREEN}"
        echo "  ShareBoxx is ready!"
        echo ""
        echo "  WiFi network:  $SSID  (open, no password)"
        echo "  Web UI:        http://${AP_IP}:3000"
        echo "  HTTPS:         https://${AP_IP}:3443"
        echo "  Stats:         http://${AP_IP}:3000/stats"
        echo ""
        echo "  Connect to '$SSID'. Any website will redirect to ShareBoxx."
        echo -e "${NC}"
    else
        warn "Some services failed to start. Check with:"
        echo "  journalctl -u hostapd -u dnsmasq -u nginx -u shareboxx --no-pager -n 30"
    fi

    cat <<INFO
Configuration files:
  hostapd:   $HOSTAPD_CONF
  dnsmasq:   $DNSMASQ_CONF
  AP setup:  $AP_UNIT  (sets ${AP_IP}/24 on $IFACE)
  nginx:     ${NGINX_SITE_CONF:-/etc/nginx/sites-available/shareboxx}
  shareboxx: /etc/systemd/system/shareboxx.service
  files:     /var/lib/shareboxx/files/

Using a USB stick for storage:

  ShareBoxx serves whatever is in /var/lib/shareboxx/files/. To use a USB
  stick (or any larger disk) as the storage backend, mount it there. Two
  common approaches:

  1) Mount the USB stick directly at /var/lib/shareboxx/files/
       sudo systemctl stop shareboxx
       lsblk                                  # find your device, e.g. sda1
       sudo mkdir -p /var/lib/shareboxx/files
       echo '/dev/sda1  /var/lib/shareboxx/files  auto  defaults,nofail,uid=shareboxx,gid=shareboxx  0 2' \\
            | sudo tee -a /etc/fstab
       sudo mount /var/lib/shareboxx/files
       sudo systemctl start shareboxx

  2) Bind-mount an existing path (e.g. when the disk is already mounted
     elsewhere like /mnt/bigdisk):
       sudo rsync -a /var/lib/shareboxx/files/ /mnt/bigdisk/shareboxx-files/
       sudo chown -R shareboxx:shareboxx /mnt/bigdisk/shareboxx-files
       echo '/mnt/bigdisk/shareboxx-files  /var/lib/shareboxx/files  none  bind  0 0' \\
            | sudo tee -a /etc/fstab
       sudo mount /var/lib/shareboxx/files

  Filesystem notes:
  - exFAT/FAT32 sticks: replace 'auto' with 'exfat' or 'vfat'. They don't
    support Unix permissions, so the uid=/gid= options are required.
  - ext4/btrfs/xfs sticks: drop the uid=/gid= options and instead chown
    once after first mount: sudo chown -R shareboxx:shareboxx /var/lib/shareboxx/files
  - 'nofail' lets the system boot even if the stick is missing; ShareBoxx
    will then serve an empty directory until you plug it back in.
INFO
}
