use mpipc::SocketType;

use crate::{AddrClaimMode, Error, get_listener};

#[test]
fn default_config_works() {
    crate::Config::try_default().unwrap();
}

// Tests:
//   - If the host supports namespaced sockets
//   - If namespaced socket listener creation fails if there is another listener present
#[test]
fn ns_connections() {
    let socket_name = "DAEMON_TEST_NS_SOCKET.socket";
    let st = SocketType::NamespacedOnly;
    let listener = get_listener(socket_name, &st, &AddrClaimMode::ClaimIfUnresponsive).unwrap();
    assert_eq!(
        get_listener(socket_name, &st, &AddrClaimMode::DoNotClaim).unwrap_err(),
        Error::SocketError
    );
    assert_eq!(
        get_listener(socket_name, &st, &AddrClaimMode::ClaimIfUnresponsive).unwrap_err(),
        Error::SocketError
    );
    assert_eq!(
        get_listener(socket_name, &st, &AddrClaimMode::ForceClaim).unwrap_err(),
        Error::SocketError
    );
    drop(listener);
}

// Tests:
//   - If filesystem socket name reclamation works properly
#[test]
fn fs_addr_reclamation() {
    let socket_name = "DAEMON_TEST_FS_SOCKET.socket";
    let st = SocketType::FilesystemOnly;
    let listener = get_listener(socket_name, &st, &AddrClaimMode::ClaimIfUnresponsive).unwrap();
    assert_eq!(
        get_listener(socket_name, &st, &AddrClaimMode::DoNotClaim).unwrap_err(),
        Error::AddrInUse
    );
    drop(listener);
    get_listener(socket_name, &st, &AddrClaimMode::ClaimIfUnresponsive).unwrap();
    get_listener(socket_name, &st, &AddrClaimMode::ForceClaim).unwrap();
}
