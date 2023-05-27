#!/bin/bash
# Copyright (c) 2023 Loki [Loki_101#3580 on Discord or loki@crazycoder.dev]

# Check if the script is run as root or with sudo
if [ "$(id -u)" != "0" ]; then
    echo "This script must be run as root or with sudo." 1>&2
    exit 1
fi

# Load database variables
if source /var/www/pterodactyl/.env; then
    echo "Loaded .env file successfully."
else
    echo "Failed to load .env file. Exiting."
    exit 1
fi

# Get "data" path from config.yml file
data_path=$(grep 'data:' /etc/pterodactyl/config.yml | awk '{print $2}')
if [ -z "$data_path" ]; then
    echo "Failed to find data path. Exiting."
    exit 1
fi

# Get the real user (even when run with sudo)
REAL_USER=${SUDO_USER:-$(whoami)}

# Split the data_path into an array of directories
IFS="/" read -ra DIRS <<< "$data_path"

# Initialize a path variable
CURRENT_PATH=""

# Iterate over each directory in the path
for DIR in "${DIRS[@]}"; do
    # Ignore empty strings (caused by leading /)
    if [ -z "$DIR" ]; then
        continue
    fi
    
    # Build the current path
    CURRENT_PATH="$CURRENT_PATH/$DIR"

    # Check if the user has read and execute access to the current directory
    if ! su - $REAL_USER -c "[ -r $CURRENT_PATH ]" || ! su - $REAL_USER -c "[ -x $CURRENT_PATH ]"; then
        while true; do
            echo "User $REAL_USER does not have read and execute access to $CURRENT_PATH. Would you like to add it? y/n"
            read answer
            if echo "$answer" | grep -iq "^y"; then
                if sudo setfacl -m u:$REAL_USER:rx $CURRENT_PATH; then
                    echo "Added $REAL_USER to ACL of $CURRENT_PATH."
                    break
                else
                    echo "Failed to set ACL. Please check if ACL is enabled on your filesystem. In most distributions the package will be called \"acl\""
                    exit 1
                fi
            elif echo "$answer" | grep -iq "^n"; then
                echo "Exiting."
                exit 1
            else
                echo "Invalid input. Please type y or n."
            fi
        done
    fi
done

# Define your MySQL command with connection details
mysql_cmd="mysql --host=$DB_HOST --port=$DB_PORT --user=$DB_USERNAME --password=$DB_PASSWORD $DB_DATABASE"

# Determine the home directory
if [ "$REAL_USER" = "root" ]; then
    home_dir="/root"
else
    home_dir="/home/$REAL_USER"
fi

# Query to fetch uuid and name pairs
query="SELECT uuid, name FROM servers;"

# Declare associative arrays to keep track of names and counts
declare -A name_count
declare -A name_freq

# Fetch the query results
results=$($mysql_cmd -N -s -r -e "$query")

# First, count the frequency of each name
while read -r uuid name; do
    # Check if UUID directory exists and it's not the .sftp directory
    if [ -d "$data_path/$uuid" ] && [ "$uuid" != ".sftp" ]; then
        # Increment the frequency count
        name_freq[$name]=$((name_freq[$name]+1))
    fi
done <<< "$results"

# Loop over results from the query again to create symlinks
while read -r uuid name; do
    # Check if UUID directory exists and it's not the .sftp directory
    if [ -d "$data_path/$uuid" ] && [ "$uuid" != ".sftp" ]; then
        # If name frequency is more than 1, append the count
        if [[ ${name_freq[$name]} -gt 1 ]]; then
            # Initialize or increment the count
            name_count[$name]=$((name_count[$name]+1))
            # Append the count to the name
            current_name="${name}${name_count[$name]}"
        else
            current_name=$name
        fi

        # Create symlink in the home directory
        if [ ! -e "$home_dir/$current_name" ]; then
            if ln -s "$data_path/$uuid" "$home_dir/$current_name"; then
                echo "Created symlink for $uuid."
            else
                echo "Failed to create symlink for $uuid."
            fi
        else
            echo "Symlink or file with the name $current_name already exists. Skipping."
        fi
    fi
done <<< "$results"