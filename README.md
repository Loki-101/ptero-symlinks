# symlinks
## Requirements
- Either Amd64 or Aarch64 architecture
- Run on the Wings machine(s)

## Install
```bash
sudo -i
export INSTALL_TO="/usr/local/share/ptero-symlinks"
```

```bash
sudo mkdir -p ${INSTALL_TO}
# Instructions are for AMD by default; if on Arm, change the ending to symlinks-aarch64-unknown-linux-musl to download the correct binary for your system
sudo wget -O ${INSTALL_TO}/symlinks https://github.com/Loki-101/ptero-symlinks/releases/latest/download/symlinks-x86_64-unknown-linux-musl
sudo chmod +x ${INSTALL_TO}/symlinks
```

**REMEMBER TO CHANGE THESE 3 VARIABLES** to match **your** environment:
```bash
sudo tee /usr/bin/symlinks >/dev/null <<'EOF'
#!/bin/bash
PANEL_FQDN="https://panel.example.com"
API_KEY="YOUR_CLIENT_API_KEY"
WINGS_CONFIG="/srv/pterodactyl/wings/config.yml"

/usr/local/share/ptero-symlinks/symlinks "$@"
EOF

sudo chmod +x /usr/local/share/ptero-symlinks/symlinks
```
You can edit this alias in case you need to change anything later with:
```bash
nano /usr/bin/symlinks
```
- In the nano text editor, you can save with Control+S, then exit with Control+X

## Notes:
- The client API key *must* be from a panel administrator account
- It is recommended to create one API key per Wings machine so you can use the "Allowed IPs" section when creating it
- Allowed IPs is from your panel's perspective; examples below:
  - Panel and Wings are on the same machine, both running normally
    - ``127.0.0.1``
  - Panel and Wings are on different machines, and the public IP of the Wings machine is "123.456.789.10"
    - ``123.456.789.10``
  - The panel in running as a Docker container, and you told it to use the subnet "172.20.0.0/24"
    - ``172.20.0.1``
  - The first address in any subnet will typically be the gateway; refer to the above example if running the panel in a Docker container, adapting it to your needs.


## Usage: ``symlinks`` if root, ``sudo symlinks`` if non-root user

## FAQ
When to run:
- Manually after server creation or deletion

Where will the symlinks be created?
- In your home directory, inside a folder called pterodactyl
- If you became root to install the binary in the default location, type ``exit`` to go back to your user before running symlinks or they will be created in root's home instead of your own.

Why are the symlinks named the way they are?
- The symlinks will be named with their human readable server name followed by a dash and their short uuid
- This is the best compromise for staying human-readable without worrying about conflicts from two servers having the same human readable name

Example:
```bash
[root@pterodactyl ~]# ls /root

[root@pterodactyl ~]# symlinks
OK — 1 symlinks ready in /root/pterodactyl
[root@pterodactyl ~]# ls /root
pterodactyl
[root@pterodactyl ~]# ls /root/pterodactyl/
'Overwatch MapVote Bot-06435f2c'
[root@pterodactyl ~]# ls /root/pterodactyl/Overwatch\ MapVote\ Bot-06435f2c/
app.py  requirements.txt
```
