# Custom
A user-friendly Discord bot that simplifies server management, automates tasks, and enhances engagement. Enjoy seamless moderation, level-based rewards, and a straightforward setup process.
## Compilation
```bash
cargo build --release --features all
./target/release/custom
```
## Feature flags
Everything that is separated by feature flags can be run separately from processes with other features, as long as they are connected to the same MongoDB and Redis databases.

| Name                | Description                                                                                                                                                                                                                                                                                                   |
| ------------------- |---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `all`               | Enables all the feature flags listed below                                                                                                                                                                                                                                                                    |
| `custom-bots`       | Listens for events on bot accounts set by users                                                                                                                                                                                                                                                               |
| `gateway`           | Listens for events on a main account ([Gateway connection]([https://discord.com/developers/docs/topics/gateway#connections](https://discord.com/developers/docs/topics/gateway#connections)))                                                                                                                 |
| `http-interactions` | Runs HTTP server on port 80 that listen for interactions sent by discord ([Receiving interactions]([https://discord.com/developers/docs/interactions/receiving-and-responding#receiving-an-interaction](https://discord.com/developers/docs/interactions/receiving-and-responding#receiving-an-interaction))) |
| `tasks`             | Runs tasks scheduler (handles actions like removing roles from temporary muted users)                                                                                                                                                                                                                         