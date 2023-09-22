# urpc
uRPC is an unreliable RPC miniframework loosely inspired by gRPC

### Creating a server
```rust
let mut server = UrpcServer::new(SRV_ADDR); 
server.register(String::from("donut-service"), DonutService::new()); 
UrpcServer::start(server);
```
### Creating the client
```rust
// Define the list of recipient nodes and instantiate the client
let recipients = vec![DONUT_ADDR];
let client = UrpcClient::new(String::from("donut-service"), recipients);


// Let's build the request
let request = DonutRequest {
  quantity: 3,
  type: "chocolate",
}

// The client encodes the request payload and sends to each recipient.
// Delivery is not guaranteed.
client.send("order", request);
```

### Further info
https://blog.lorisocchipinti.com/rust-urpc/
