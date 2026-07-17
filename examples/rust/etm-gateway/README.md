# ETM Gateway PoC

First the server needs to run
```sh
cargo run --example ping-pong-server --release
cargo run --example pong --release
```

Then run the client
```sh
cargo run --example ping-pong-client --release
cargo run --example ping --release
```

If the server is on a different host, pass the server IP to the client
```sh
cargo run --example ping-pong-client --release -- --server 192.168.1.92
cargo run --example ping --release
```

NOTE: the port 0xA2D2 (41682) is used to request a connection from the server and then the server assigns a free port to the client. A firewall might prevent the communication.
