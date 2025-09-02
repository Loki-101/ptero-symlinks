# ⚠️V6 currently under maintenance; continue using V5 until this warning is removed⚠️

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
sudo tee /usr/bin/symlinks >/dev/null <<EOF
#!/bin/bash
PANEL_FQDN="https://panel.example.com"
API_KEY="YOUR_CLIENT_API_KEY"
WINGS_CONFIG="/srv/pterodactyl/wings/config.yml"

${INSTALL_TO}/symlinks "$@"
EOF

sudo chmod +x /usr/bin/symlinks
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

## Examples
#### root
```bash
node1 login: root
Password: 
[root@node1 ~]# export INSTALL_TO="/usr/local/share/ptero-symlinks"
[root@node1 ~]# sudo mkdir -p ${INSTALL_TO}
[root@node1 ~]# sudo wget -O ${INSTALL_TO}/symlinks https://github.com/Loki-101/ptero-symlinks/releases/latest/download/symlinks-x86_64-unknown-linux-musl
--2025-09-02 02:58:37--  https://github.com/Loki-101/ptero-symlinks/releases/latest/download/symlinks-x86_64-unknown-linux-musl
Resolving github.com (github.com)... 140.82.116.3
Connecting to github.com (github.com)|140.82.116.3|:443... connected.
HTTP request sent, awaiting response... 302 Found
Location: https://github.com/Loki-101/ptero-symlinks/releases/download/V6/symlinks-x86_64-unknown-linux-musl [following]
--2025-09-02 02:58:38--  https://github.com/Loki-101/ptero-symlinks/releases/download/V6/symlinks-x86_64-unknown-linux-musl
Reusing existing connection to github.com:443.
HTTP request sent, awaiting response... 302 Found
Location: https://release-assets.githubusercontent.com/github-production-release-asset/646006725/bab30455-e13e-494b-ac10-a4206f74a3cb?sp=r&sv=2018-11-09&sr=b&spr=https&se=2025-09-02T03%3A41%3A00Z&rscd=attachment%3B+filename%3Dsymlinks-x86_64-unknown-linux-musl&rsct=application%2Foctet-stream&skoid=96c2d410-5711-43a1-aedd-ab1947aa7ab0&sktid=398a6654-997b-47e9-b12b-9515b896b4de&skt=2025-09-02T02%3A40%3A10Z&ske=2025-09-02T03%3A41%3A00Z&sks=b&skv=2018-11-09&sig=fmJnDznmZzcJBvN7wHyh4KSU42zIEDm8mZdjR%2BUc7eA%3D&jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmVsZWFzZS1hc3NldHMuZ2l0aHVidXNlcmNvbnRlbnQuY29tIiwia2V5Ijoia2V5MSIsImV4cCI6MTc1Njc4MjIxOCwibmJmIjoxNzU2NzgxOTE4LCJwYXRoIjoicmVsZWFzZWFzc2V0cHJvZHVjdGlvbi5ibG9iLmNvcmUud2luZG93cy5uZXQifQ.IpDz1IrR9N3UWU4PevUoMy5Q4jzxt4n-aa0r2cfadRA&response-content-disposition=attachment%3B%20filename%3Dsymlinks-x86_64-unknown-linux-musl&response-content-type=application%2Foctet-stream [following]
--2025-09-02 02:58:38--  https://release-assets.githubusercontent.com/github-production-release-asset/646006725/bab30455-e13e-494b-ac10-a4206f74a3cb?sp=r&sv=2018-11-09&sr=b&spr=https&se=2025-09-02T03%3A41%3A00Z&rscd=attachment%3B+filename%3Dsymlinks-x86_64-unknown-linux-musl&rsct=application%2Foctet-stream&skoid=96c2d410-5711-43a1-aedd-ab1947aa7ab0&sktid=398a6654-997b-47e9-b12b-9515b896b4de&skt=2025-09-02T02%3A40%3A10Z&ske=2025-09-02T03%3A41%3A00Z&sks=b&skv=2018-11-09&sig=fmJnDznmZzcJBvN7wHyh4KSU42zIEDm8mZdjR%2BUc7eA%3D&jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmVsZWFzZS1hc3NldHMuZ2l0aHVidXNlcmNvbnRlbnQuY29tIiwia2V5Ijoia2V5MSIsImV4cCI6MTc1Njc4MjIxOCwibmJmIjoxNzU2NzgxOTE4LCJwYXRoIjoicmVsZWFzZWFzc2V0cHJvZHVjdGlvbi5ibG9iLmNvcmUud2luZG93cy5uZXQifQ.IpDz1IrR9N3UWU4PevUoMy5Q4jzxt4n-aa0r2cfadRA&response-content-disposition=attachment%3B%20filename%3Dsymlinks-x86_64-unknown-linux-musl&response-content-type=application%2Foctet-stream
Resolving release-assets.githubusercontent.com (release-assets.githubusercontent.com)... 185.199.111.133, 185.199.108.133, 185.199.109.133, ...
Connecting to release-assets.githubusercontent.com (release-assets.githubusercontent.com)|185.199.111.133|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: 3257576 (3.1M) [application/octet-stream]
Saving to: ‘/usr/local/share/ptero-symlinks/symlinks’

/usr/local/share/p 100%[=============>]   3.11M  --.-KB/s    in 0.1s    

2025-09-02 02:58:38 (31.5 MB/s) - ‘/usr/local/share/ptero-symlinks/symlinks’ saved [3257576/3257576]

[root@node1 ~]# sudo chmod +x ${INSTALL_TO}/symlinks
[root@node1 ~]# sudo tee /usr/bin/symlinks >/dev/null <<EOF
#!/bin/bash
export PANEL_FQDN="https://panel.example.com"
export API_KEY="ptlc_REDACTEDv1mPfJ1"
export WINGS_CONFIG="/srv/pterodactyl/wings/config.yml"

${INSTALL_TO}/symlinks "$@"
EOF
[root@node1 ~]# sudo chmod +x /usr/bin/symlinks
[root@node1 ~]# ls

[root@node1 ~]# symlinks
OK — 1 symlinks ready in /root/pterodactyl
[root@node1 ~]# ls
app.py  pterodactyl  run.sh  videos
[root@node1 ~]# ls pterodactyl/
'Overwatch MapVote Bot-06435f2c'
[root@node1 ~]# ls pterodactyl/Overwatch\ MapVote\ Bot-06435f2c/
app.py  requirements.txt
```
#### non-root user
```bash
node1 login: testuser
Password: 
[testuser@node1 ~]$ sudo -i
[sudo] password for testuser: 
[root@node1 ~]# export INSTALL_TO="/usr/local/share/ptero-symlinks"
[root@node1 ~]# sudo mkdir -p ${INSTALL_TO}
[root@node1 ~]# sudo wget -O ${INSTALL_TO}/symlinks https://github.com/Loki-101/ptero-symlinks/releases/latest/download/symlinks-x86_64-unknown-linux-musl
--2025-09-02 06:27:09--  https://github.com/Loki-101/ptero-symlinks/releases/latest/download/symlinks-x86_64-unknown-linux-musl
Resolving github.com (github.com)... 140.82.116.3
Connecting to github.com (github.com)|140.82.116.3|:443... connected.
HTTP request sent, awaiting response... 302 Found
Location: https://github.com/Loki-101/ptero-symlinks/releases/download/V6/symlinks-x86_64-unknown-linux-musl [following]
--2025-09-02 06:27:10--  https://github.com/Loki-101/ptero-symlinks/releases/download/V6/symlinks-x86_64-unknown-linux-musl
Reusing existing connection to github.com:443.
HTTP request sent, awaiting response... 302 Found
Location: https://release-assets.githubusercontent.com/github-production-release-asset/646006725/4f32efad-cb14-4450-985b-1a52f60b05a9?sp=r&sv=2018-11-09&sr=b&spr=https&se=2025-09-02T07%3A22%3A07Z&rscd=attachment%3B+filename%3Dsymlinks-x86_64-unknown-linux-musl&rsct=application%2Foctet-stream&skoid=96c2d410-5711-43a1-aedd-ab1947aa7ab0&sktid=398a6654-997b-47e9-b12b-9515b896b4de&skt=2025-09-02T06%3A21%3A54Z&ske=2025-09-02T07%3A22%3A07Z&sks=b&skv=2018-11-09&sig=vx9JQDJu5QgPKr8LHT3%2FJ97WWvGPVEw9q3qvDeVRCV4%3D&jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmVsZWFzZS1hc3NldHMuZ2l0aHVidXNlcmNvbnRlbnQuY29tIiwia2V5Ijoia2V5MSIsImV4cCI6MTc1Njc5NDczMCwibmJmIjoxNzU2Nzk0NDMwLCJwYXRoIjoicmVsZWFzZWFzc2V0cHJvZHVjdGlvbi5ibG9iLmNvcmUud2luZG93cy5uZXQifQ.F2sGbcBePwIsX9wYRbc9HgO5nvir2gASrm-x94lVX0U&response-content-disposition=attachment%3B%20filename%3Dsymlinks-x86_64-unknown-linux-musl&response-content-type=application%2Foctet-stream [following]
--2025-09-02 06:27:10--  https://release-assets.githubusercontent.com/github-production-release-asset/646006725/4f32efad-cb14-4450-985b-1a52f60b05a9?sp=r&sv=2018-11-09&sr=b&spr=https&se=2025-09-02T07%3A22%3A07Z&rscd=attachment%3B+filename%3Dsymlinks-x86_64-unknown-linux-musl&rsct=application%2Foctet-stream&skoid=96c2d410-5711-43a1-aedd-ab1947aa7ab0&sktid=398a6654-997b-47e9-b12b-9515b896b4de&skt=2025-09-02T06%3A21%3A54Z&ske=2025-09-02T07%3A22%3A07Z&sks=b&skv=2018-11-09&sig=vx9JQDJu5QgPKr8LHT3%2FJ97WWvGPVEw9q3qvDeVRCV4%3D&jwt=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmVsZWFzZS1hc3NldHMuZ2l0aHVidXNlcmNvbnRlbnQuY29tIiwia2V5Ijoia2V5MSIsImV4cCI6MTc1Njc5NDczMCwibmJmIjoxNzU2Nzk0NDMwLCJwYXRoIjoicmVsZWFzZWFzc2V0cHJvZHVjdGlvbi5ibG9iLmNvcmUud2luZG93cy5uZXQifQ.F2sGbcBePwIsX9wYRbc9HgO5nvir2gASrm-x94lVX0U&response-content-disposition=attachment%3B%20filename%3Dsymlinks-x86_64-unknown-linux-musl&response-content-type=application%2Foctet-stream
Resolving release-assets.githubusercontent.com (release-assets.githubusercontent.com)... 185.199.110.133, 185.199.111.133, 185.199.109.133, ...
Connecting to release-assets.githubusercontent.com (release-assets.githubusercontent.com)|185.199.110.133|:443... connected.
HTTP request sent, awaiting response... 200 OK
Length: 3270976 (3.1M) [application/octet-stream]
Saving to: ‘/usr/local/share/ptero-symlinks/symlinks’

/usr/local/share/p 100%[=============>]   3.12M  --.-KB/s    in 0.1s    

2025-09-02 06:27:10 (25.6 MB/s) - ‘/usr/local/share/ptero-symlinks/symlinks’ saved [3270976/3270976]

[root@node1 ~]# sudo chmod +x ${INSTALL_TO}/symlinks
[root@node1 ~]# sudo tee /usr/bin/symlinks >/dev/null <<EOF
#!/bin/bash
export PANEL_FQDN="https://panel.example.com"
export API_KEY="ptlc_REDACTEDv1mPfJ1"
export WINGS_CONFIG="/srv/pterodactyl/wings/config.yml"

${INSTALL_TO}/symlinks "$@"
EOF
[root@node1 ~]# sudo chmod +x /usr/bin/symlinks
[root@node1 ~]# exit
logout
[testuser@node1 ~]$ ls

[testuser@node1 ~]$ symlinks 
must run as root/sudo
[testuser@node1 ~]$ sudo symlinks
note: 'testuser' is not in group 'pterodactyl' (gid 988).
Add now and set ACL permissions for the group? [y/N] y
✔ added 'testuser' to group 'pterodactyl'. You may need to re-login or run 'newgrp pterodactyl' to apply it.
OK — 1 symlinks ready in /home/testuser/pterodactyl
[testuser@node1 ~]$ ls
pterodactyl
[testuser@node1 ~]$ groups
testuser wheel
[testuser@node1 ~]$ newgrp pterodactyl
[testuser@node1 ~]$ groups
pterodactyl testuser wheel
[testuser@node1 ~]$ ls pterodactyl/
'Overwatch MapVote Bot-06435f2c'
[testuser@node1 ~]$ ls pterodactyl/Overwatch\ MapVote\ Bot-06435f2c/
app.py  requirements.txt
```
