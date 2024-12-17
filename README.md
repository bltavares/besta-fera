# Besta Fera
Discord bot to manage Minecraft servers on my [homelab](https://github.com/bltavares/homelab).

## Available commands
- `/start <server>`: Start a server
- `/stop <server>`: Stop a server
- `/status`: Get the status of a server

## Installation

### Discord integration
Open the Discord Developer Portal and create a new bot. Use the token to configure the bot.

Enable on OAuth2 the `bot` scope and `applications.commands` permissions: 
  - Slash Commands
  - Send Messages
  - 
Then paste the link on the channel to invite it to the server.

### Running

Requires access to the `docker.sock` to be able to start and stop containers.

```bash
# Compile
cargo build --release
sudo mv target/release/besta-fera /usr/local/bin

# Store Discord Token
sudo sh -c 'echo "DISCORD_TOKEN=your_token" > /etc/default/besta-fera'

# Enable Daemon
sudo cp besta-fera.service /etc/systemd/system
sudo systemctl daemon-reload
sudo systemctl enable besta-fera
sudo systemctl start besta-fera
```

Or use the provided Docker image:
```bash
docker run -d --name besta-fera \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -e DISCORD_TOKEN=your_token \
    --restart unless-stopped \
    ghcr.io/bltavares/besta-fera:latest
```
