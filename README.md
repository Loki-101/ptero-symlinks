# WARNING: THIS SCRIPT ASSUMES /var/www/pterodactyl/.env AND /etc/pterodactyl/config.yml ARE IN THOSE LOCATIONS.
### This means the script will not function with the docker-compose.yml version of the panel since it doesn't have a .env file in that location, and it also will not function when your panel is on a different machine than your node. It won't break anything; it will just throw an error and exit.

# ptero-symlinks
Automatically creates symlinks for Pterodactyl Servers in the user's home directory based on the name in the Panel

# Dependencies: The acl package is required if running this as a normal user:
- RHEL Base: `dnf -y install acl`
- Debian Base: `apt -y install acl`

# Download from Linux Command Line
```bash
wget https://github.com/Loki-101/ptero-symlinks/releases/download/latest/symlinks.sh
chmod +x symlinks.sh
```

# Usage as a normal user:
sudo ./symlinks.sh

![user](https://github.com/Loki-101/ptero-symlinks/assets/59907407/517f0be6-4dc8-43c4-9136-fd44271c1613)

# Usage as root:
./symlinks.sh

![root](https://github.com/Loki-101/ptero-symlinks/assets/59907407/41f18113-4a9e-40bd-be95-0419cd4d9d2f)

# End result:
In your home folder, so depending on who you ran the script on either in /root or in /home/your-user you will now have folders with human readable names for all your Pterodactyl Panel servers. If you have two servers with the same name a 1 will be appended to the first one, and it will keep increasing. For example, Redbot1 and Redbot2 symlinks will be created if you have two servers named Redbot.
