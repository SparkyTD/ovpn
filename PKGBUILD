# Maintainer: Your Name <youremail@example.com>
pkgname=ovpn
pkgver=1.0.0
pkgrel=1
pkgdesc="A daemon and CLI for managing OpenVPN connections"
arch=('x86_64')
url="https://yourprojecturl.com"
license=('MIT' 'Apache-2.0')
depends=('openssl')
makedepends=('cargo')

build() {
    cd "$srcdir/.."
    # Build all workspace members in release mode
    cargo build --release --workspace
}

package() {
    cd "$srcdir"

    # Install the ovpnd daemon
    install -Dm755 "$srcdir/../target/release/ovpnd" "$pkgdir/usr/bin/ovpnd"

    # Install the ovpn-cli tool
    install -Dm755 "$srcdir/../target/release/ovpn-cli" "$pkgdir/usr/bin/ovpn-cli"

    # Install the systemd service file
    install -Dm644 "$srcdir/../ovpnd.service" "$pkgdir/usr/lib/systemd/system/ovpnd.service"

    # Ensure the runtime directory exists (handled by systemd via the service file)
    # No need to create /run/ovpnd-daemon.sock here; it's created by the daemon at runtime
}

# No post_install or pre_install functions are necessary
