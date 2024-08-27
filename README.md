# ptero-symlinks
Automatically creates symlinks for Pterodactyl Servers in the user's home directory based on their human readable name
- Does not require the panel do be on the same machine as wings
- Should be run from the WINGS machine
- If panel and wings are on different machines or your .env file in is anywhere other than /var/www/pterodactyl/.env, this script will ask you if you want to specify the path to the .env file or enter the database connection info manually.
- Compatible with running your panel inside a compose stack - just enter the info manually and make sure your user is allowed to connect from the host machine. (Out of box compatibility is planned for the next large block of free time I get)

# Dependencies:
## ACL (Access Control List)
- RHEL Base: `dnf -y install acl`
- Debian Base: `apt -y install acl`
## MariaDB or MySQL Client
- The package names differ more than acl in various distributions, but it will usually be along the lines of mariadb-client or mysql-client.

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
![image](https://github.com/Loki-101/ptero-symlinks/assets/59907407/79cbf8f7-a948-4bf2-a465-ff0882deccf2)

