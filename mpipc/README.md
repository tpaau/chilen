# Music Player Interprocess Communication library

<small>This library is in no way related to [MPD](https://www.musicpd.org/).</small>

> ### Warning
> The project is under active development, so please be wary of any bugs you may find, and expect
> breaking API changes in the upcoming updates!

This library provides common data types and functions used to communicate with the music player
daemon. No additional crates are required to create a simple client.

Under the hood, `mpipc` uses [`interprocess`] and [`rmp_serde`] to talk to the `daemon` on a local
socket. The type of socket used depends on your platform and the startup configuration of the
`daemon`.

The `daemon` communicates by listening for [commands](Command), dispatching them, and sending back
[responses](Response). It additionally features a special command, [`Command::EventStream`]. Running
it causes the `daemon` start sending [events](Event) to the client on the same connection whenever
there is an important state change, eg. the music library content changed, the player was paused,
or the track queue changed.

# Examples

**Note:** all the examples provided require a running `daemon` instance to work.

Send a single command to a running `daemon`, get the response, and disconnect:
```no_run
# use mpipc::{send_command, Command, DEFAULT_SOCKET_NAME, SocketType, Response};
assert_eq!(
  send_command(
    Command::Ping,
    DEFAULT_SOCKET_NAME,
    &SocketType::default()
  ).unwrap(),
  Response::Pong
)
```

Connect to a running `daemon`, send a command and then disconnect:
```no_run
  # use std::io::{BufReader, Write};
  # use mpipc::{connect, DEFAULT_SOCKET_NAME, SocketType, serialize_client_command, Command, Error, Response, disconnect, receive_response};
  let conn = connect(
    // The socket the `daemon` listens on. A default socket address is provided in `mpipc`, however,
    // you shouldn't use it outside of testing
    DEFAULT_SOCKET_NAME,
    // The type of IPC socket `daemon` listens on
    &SocketType::default()
  ).unwrap();
  let mut conn = BufReader::new(conn);

  // Serialize and send the command to the `daemon`
  let cmd = mpipc::serialize_client_command(&Command::Ping).unwrap();
  conn.get_mut().write_all(&cmd).unwrap();

  // The `daemon` will always respond to `Command::Ping` with `Response::Pong`
  assert_eq!(receive_response(&mut conn).unwrap(), Response::Pong);

  // Close the connection
  disconnect(&mut conn).unwrap();
```
