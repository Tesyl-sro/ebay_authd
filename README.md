# Ebay API Token Daemon

Automatically fetches the latest Ebay API token and provides an IPC using UNIX sockets.

Usage:
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

### Note
This program cannot be used as a service, as the start command requires manual authentication.