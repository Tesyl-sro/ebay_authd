# Ebay API Token Daemon

Automatically fetches the latest Ebay API token and provides an IPC using UNIX sockets.
The socket provides a JSON-based messaging protocol for communication. All JSON messages must end with a newline (`\n`) character.

The UNIX socket is created in `/tmp/ebay_authd.sock`.

### Notes
This program cannot be used as a service, as the start command requires manual authentication.

### Compatibility
- [x] Linux
- [x] macOS
- [ ] Windows

### Usage:
```
Usage: ebay_authd <COMMAND>

Commands:
  daemon  Daemon control commands
  test    Testing commands
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
```
Daemon control commands

Usage: ebay_authd daemon <COMMAND>

Commands:
  start   Start the daemon
  status  Get the status of the daemon
  reauth  Fix a broken daemon instance
  stop    Ask the daemon to stop
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
```
Testing commands

Usage: ebay_authd test <COMMAND>

Commands:
  token   Get the latest token
  status  Get the status of the daemon
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Testing
Use the testing commands to verify that the daemon is working.

### 3rd party access
You can use other programs to communicate with the daemon, like `socat`:
```sh
# Request the latest token (echo adds a newline)
echo "{\"Request\": \"Token\"}" | socat - UNIX-CONNECT:/tmp/ebay_authd.sock
```
```sh
# Stop the daemon
echo "{\"Request\": \"Stop\"}" | socat - UNIX-CONNECT:/tmp/ebay_authd.sock
```

For more detailed information, read the individual READMEs in `ebay_authd_core` and `ebay_authd_client`.