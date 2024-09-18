## Simple OpenVPN Client Session Manager
This project aims to provide a very basic session manager daemon for your OpenVPN connections, written entirely in Rust. At this stage, the daemon works perfectly fine, but as it's my first Rust project, I would not recommend it for production use.

Also check out my Hyprland [dotfiles](https://github.com/SparkyTD/dotfiles), which include a custom GUI module for the [AGS bar](https://github.com/Aylur/ags), that natively integrates with this project.
___

### Feature list / TODO
- [x] Importing `.ovpn` config files
- [x] Exporting configurations
- [x] Managing multiple configurations
- [x] Starting and stopping connections
- [x] Querying info about the currently active connection (if any)
- [ ] Multiple simultaneous connections
- [ ] Dedicated OpenVPN client log management
- [x] UNIX Socket based interface
- [x] CLI interface
- [x] Real-time session start/stop events via the UNIX Sokcet
- [ ] Configurabilty
  - [ ] Custom socket file path
  - [ ] Custom socket file permissions (currently hard-coded to 777)
  - [ ] Startup behavior (e.g. automatically start the most recent session)
  - [ ] Custom openvpn executable path (currently hard-coded to "/usr/bin/openvpn")
  - [ ] Custom log path

___

### Installation (Arch-based distros)
1. Clone this repository: `$ git clone https://github.com/SparkyTD/ovpn`
2. Enter the directory: `$ cd ovpn`
3. Build and install the package: `$ makepkg -si`

### Installation (All other distros)
This project has only been tested on Arch, but it can be used on any other distro based on systemd, as long as you have `rust` and `openvpn` installed.
1. Clone the repository and enter the directory (same as on Arch)
2. Compile the Rust project: `cargo build -r`
3. Copy the compiled binaries to your preferred folder (`./target/release/ovpnd` and `./target/release/ovpn-cli`), or add the `./target/release/` fodler to your `$PATH`
4. Use the included `ovpnd.service` file as a template to integrate the daemon as a systemd service.

___

### Usage (via the CLI)
**Start the service** 
```
# systemctl enable --now ovpnd
```
**Import a configuration**
```
$ ovpn-cli config import --name my_company --path /home/me/Downloads/corp_vpn.ovpn
```
**Start a session**
```
$ ovpn-cli session start --name my_company
```

___

### Usage (via the UNIX Socket)
The socket is located at `/run/ovpnd-daemon.sock`, and is accessible for all non-root users by default (currently there is no way to change this). Once a client connects to the socket, it accepts the same commands as the CLI tool. You can find out more about the commands by running `ovpn-cli` or `ovpn-cli [subcommand]`. 

After each command, the socket will respond in the following format: `<length>:<status>:<message>`, where `<length>` represents the total length of `<status>:<message>`, `<status>` is one of "ok" or "err", and `<message>` is the full response message (can be multi-line, `<length>` includes newline characters) or an error message.

When the status of a session changes (e.g. started / stopped), the socket will broadcast an event message to all connected clients. Keep in mind that this may happen *while* a command response is being written, and they should be ignored when parsing a multi-line response.

Event messages use the following format: `!<length>:<guid>:<name>:<status>` where `<length>` encodes the length of `<guid>:<name>:<status>`, and the rest of the parameters contain information about the session status change. The `<status>` parameter can be one of: `Starting`, `Running`, `Stopping` or `Stopped` depending on the event type. Note the exclamation point at the start, which indicates that this is an event broadcast.

___

### Legal disclaimer
THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
