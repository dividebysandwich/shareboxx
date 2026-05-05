#!/usr/bin/env bash
#
# ShareBoxx — single-script installer for non-Debian / source builds.
#
# Does the full installation in one go:
#   - Installs the freshly built binary and frontend assets
#   - Creates the shareboxx system user
#   - Installs and enables the systemd service
#   - Configures hostapd, dnsmasq, dhcpcd, iptables (captive portal)
#   - Configures nginx with a self-signed certificate (HTTPS)
#
# The WiFi network is open by design — ShareBoxx is meant to be passwordless.
#
# Run AFTER `cargo leptos build --release`, from the repository root or from
# the `access-point/` directory:
#
#   sudo ./access-point/install-from-source.sh             # install + configure
#   sudo ./access-point/install-from-source.sh --uninstall # revert AP config
#
# Required commands (install via your distro's package manager first):
#   hostapd, dnsmasq, dhcpcd, nginx, openssl, iw, iptables, rfkill, systemctl
#
set -euo pipefail

# ── CLI ─────────────────────────────────────────────────────────────────────

usage() {
    cat <<USAGE
Usage: install-from-source.sh [--uninstall] [--help]

Without flags, runs the full source install + interactive AP setup.

  --uninstall    Revert the access-point configuration (dhcpcd block, dnsmasq,
                 hostapd, iptables rules, NetworkManager unmanage rule, nginx
                 site). Does NOT remove the binary, the user, or files in
                 /var/lib/shareboxx — for that, undo manually.
  -h, --help     Show this help.
USAGE
}

DO_UNINSTALL=0
for arg in "$@"; do
    case "$arg" in
        --uninstall) DO_UNINSTALL=1 ;;
        -h|--help)   usage; exit 0 ;;
        *) echo "Unknown argument: $arg" >&2; usage >&2; exit 2 ;;
    esac
done

# ── Source shared library ───────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [[ ! -f "$SCRIPT_DIR/setup-lib.sh" ]]; then
    echo "ERROR: setup-lib.sh missing next to this script ($SCRIPT_DIR/setup-lib.sh)" >&2
    exit 1
fi
# shellcheck disable=SC1091
. "$SCRIPT_DIR/setup-lib.sh"

# ── Preflight ───────────────────────────────────────────────────────────────

if [[ $EUID -ne 0 ]]; then
    err "This script must be run as root."
    exit 1
fi

if [[ $DO_UNINSTALL -eq 1 ]]; then
    do_uninstall
    info "Binary, shareboxx user, systemd service, and /var/lib/shareboxx are untouched."
    exit 0
fi

# ── Required commands ──────────────────────────────────────────────────────

step "Checking required commands"

REQUIRED_CMDS=(hostapd dnsmasq nginx openssl iw ip iptables systemctl)
MISSING=()
for cmd in "${REQUIRED_CMDS[@]}"; do
    command -v "$cmd" &>/dev/null || MISSING+=("$cmd")
done
if [[ ${#MISSING[@]} -gt 0 ]]; then
    err "Missing required commands: ${MISSING[*]}"
    cat <<HINT
Install them with your distro's package manager, e.g.:
  Arch:    pacman -S hostapd dnsmasq nginx openssl iw iproute2 iptables
  Fedora:  dnf install hostapd dnsmasq nginx openssl iw iproute iptables iptables-services
  openSUSE: zypper install hostapd dnsmasq nginx openssl iw iproute2 iptables
  Debian:  apt install hostapd dnsmasq nginx openssl iw iproute2 iptables \\
                       netfilter-persistent iptables-persistent
HINT
    exit 1
fi
ok "All required commands present"

# ── Build artifacts ─────────────────────────────────────────────────────────

step "Checking build artifacts"

BIN_SRC="$REPO_ROOT/target/release/shareboxx"
SITE_SRC="$REPO_ROOT/target/site"

if [[ ! -x "$BIN_SRC" ]]; then
    err "Binary not found at $BIN_SRC"
    err "Build first: cargo leptos build --release"
    exit 1
fi
if [[ ! -d "$SITE_SRC" ]]; then
    err "Frontend assets not found at $SITE_SRC"
    err "Build first: cargo leptos build --release"
    exit 1
fi
ok "Found binary and frontend assets"

# ── Setup flow ──────────────────────────────────────────────────────────────

select_iface
check_ap_capability "$IFACE"
check_service_conflicts "$IFACE"
prompt_config

# Install binary + assets + user + systemd service (source-specific).
step "Installing binary and frontend assets"

install -d -m 755 /var/lib/shareboxx/files /var/lib/shareboxx/site
install -m 755 "$BIN_SRC" /usr/bin/shareboxx
cp -r "$SITE_SRC"/* /var/lib/shareboxx/site/

if ! getent group shareboxx >/dev/null 2>&1; then
    groupadd --system shareboxx
fi
if ! getent passwd shareboxx >/dev/null 2>&1; then
    useradd --system --gid shareboxx --home-dir /var/lib/shareboxx \
            --no-create-home --shell /usr/sbin/nologin shareboxx
fi
chown -R shareboxx:shareboxx /var/lib/shareboxx
chmod 755 /var/lib/shareboxx/files
ok "Installed shareboxx binary and assets"

step "Installing systemd service"
cat > /etc/systemd/system/shareboxx.service <<SERVICE
[Unit]
Description=Shareboxx Service
After=network.target

[Service]
WorkingDirectory=/var/lib/shareboxx
ExecStart=/usr/bin/shareboxx
Restart=on-failure
RestartSec=5
User=shareboxx
Group=shareboxx
Environment=LEPTOS_OUTPUT_NAME=shareboxx
Environment=LEPTOS_SITE_ROOT=/var/lib/shareboxx/site
Environment=LEPTOS_SITE_PKG_DIR=pkg
Environment=LEPTOS_SITE_ADDR=0.0.0.0:3000
Environment=LEPTOS_RELOAD_PORT=3001

[Install]
WantedBy=multi-user.target
SERVICE
systemctl daemon-reload
systemctl enable shareboxx.service
ok "shareboxx.service installed and enabled"

systemctl stop dnsmasq   2>/dev/null || true
systemctl stop hostapd   2>/dev/null || true
systemctl stop shareboxx 2>/dev/null || true

configure_ap_interface
configure_dnsmasq
configure_hostapd
configure_iptables_redirect
configure_nginx_ssl

start_services_and_check
print_summary

cat <<INFO

To reconfigure: re-run this script.
To revert:      sudo $(basename "$0") --uninstall
To check:       systemctl status shareboxx
INFO
