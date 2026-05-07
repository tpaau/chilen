# Chilen Inter-Process Communication library

> ### Warning
> The project is under active development, so please be wary of any bugs you may find, and expect
> breaking API changes in the upcoming updates!

This library provides common data types and functions used to communicate with the chilen daemon.
No additional crates are required to create a simple client.

Under the hood, [`chilen_ipc`](crate) uses [`interprocess`] and [`rmp_serde`] to talk to the chilen
daemon on a local socket. The type of socket used depends on your platform and the startup
configuration of the chilen daemon.

The daemon communicates by listening for [commands](Command), dispatching them, and sending back
[responses](Response). It can also stream [events](Event) to the client whenever there is an
important state change, eg. the music library content changed, the player was paused, or the track
queue changed.

# Examples

**Note:** all the examples provided require a running daemon instance to work.

Send a single command to a running daemon, get the response, and disconnect:
```no_run
# use chilen_ipc::{send_command, Command, DEFAULT_SOCKET_NAME, SocketType, Response};
assert_eq!(
  send_command(
    Command::Ping,
    DEFAULT_SOCKET_NAME,
    &SocketType::default()
  ).unwrap(),
  Response::Pong
)
```

Connect to a running daemon, send a command and then disconnect:
```no_run
  # use std::io::{BufReader, Write};
  # use chilen_ipc::{connect, DEFAULT_SOCKET_NAME, SocketType, serialize_command, Command, Error, Response, disconnect, receive_response};
  let conn = connect(
    // The socket the daemon listens on. Default socket address is provided in `chilen_ipc`, but
    // it shouldn't be used outside of testing
    DEFAULT_SOCKET_NAME,
    // The type of IPC socket daemon listens on
    &SocketType::default()
  ).unwrap();
  let mut conn = BufReader::new(conn);

  // Serialize and send the command to the daemon
  let cmd = chilen_ipc::serialize_command(&Command::Ping).unwrap();
  conn.get_mut().write_all(&cmd).unwrap();

  // The daemon will always respond to `Command::Ping` with `Response::Pong`
  assert_eq!(receive_response(&mut conn).unwrap(), Response::Pong);

  // Close the connection
  disconnect(&mut conn).unwrap();
```
