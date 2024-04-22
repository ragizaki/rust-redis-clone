[![progress-banner](https://backend.codecrafters.io/progress/redis/c8e803e0-7f95-44cf-97dd-591e9f6c1118)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is my implementation of CodeCrafter's
["Build Your Own Redis" Challenge](https://app.codecrafters.io/courses/redis/introduction).

Implemented using Rust. The server is able to handle concurrent Redis connections using Tokio threads and async file, read and write operations. Commands are supported via Redis' serialization protocol (RESP) specification. You can read more about the spec [here])(https://redis.io/docs/latest/develop/reference/protocol-spec/)

## Master/Replica Architecture
![master/replica architecture](https://miro.medium.com/v2/resize:fit:1062/1*LkgG8SiU3pbeslElStmY9w.png)

My implementation uses the `tokio::mpsc::channel` crate to broadcast changes from a master to all its replicas running on separate coroutines via Tokio's runtime. The server supports syncing via an RDB file. Each server has a data store allowing the user to `set` and `get` different cache values.
