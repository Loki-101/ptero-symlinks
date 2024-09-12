# ptero-symlinks
Automatically creates symlinks for Pterodactyl Servers in the user's home directory based on their human readable name
- Should be run from the WINGS machine; if the panel is not hosted on the same machine, you will have to enter the information to connect to your panel's database manually. Once you'd verified it works, you can create a file at /var/www/pterodactyl/.env with the following variables filled out so you don't have to enter it manually each time (The database will have to be open to outside connections from your Wings machine's IP; you can always make a new user for this with read access on the panel database):
  - ```
    DB_HOST=
    DB_PORT=
    DB_USERNAME=
    DB_DATABASE=
    DB_PASSWORD=""
    ```
- Compatible with running your panel inside a compose stack if run in the same directory as your panel's docker-compose.yml file or if the compose file is located at /srv/pterodactyl/docker-compose.yml

# Dependencies:
## ACL (Access Control List)
- RHEL Base: `dnf -y install acl`
- Debian Base: `apt -y install acl`
## MariaDB or MySQL Client
- The package names differ more than acl in various distributions, but it will usually be along the lines of mariadb-client or mysql-client.

# Download from Linux Command Line
```bash
wget https://raw.githubusercontent.com/Loki-101/ptero-symlinks/main/symlinks.sh
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

