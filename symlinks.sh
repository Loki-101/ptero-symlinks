#!/bin/bash
# Copyright (c) 2023-present Loki [loki_101 on Discord or loki@crazycoder.dev]

set -e

# Define global variables
PERMISSION_FIX_AGREED=0
DOCKER_DETECTED=0
COMPOSE_FILE=""

# Check if the script is run as root or with sudo
if [ "$(id -u)" != "0" ]; then
    echo "This script must be run as root or with sudo." 1>&2
    exit 1
fi

# Function to extract database info from docker-compose.yml
extract_db_info_from_compose() {
    local compose_file=$1
    local env_file=$(dirname "$compose_file")/.env

    # Source the .env file if it exists
    if [ -f "$env_file" ]; then
        set -a
        source <(grep -v '^#' "$env_file" | sed 's/\r$//' | sed 's/^export //' | sed 's/=\(.*\)/="\1"/')
        set +a
    fi

    # Extract database info from docker-compose.yml
    DB_DATABASE=$(grep 'MYSQL_DATABASE:' "$compose_file" | awk '{print $2}' | tr -d '"')
    DB_USERNAME=$(grep 'MYSQL_USER:' "$compose_file" | awk '{print $2}' | tr -d '"')

    # Set default values for host and port
    DB_HOST="127.0.0.1"
    DB_PORT="3306"

    DOCKER_DETECTED=1
    COMPOSE_FILE=$compose_file
}

# Function to find Wings config path in docker-compose.yml
find_wings_config_path() {
    local compose_file=$1
    local wings_path=$(grep -oP '(?<=- ").*(?=:/etc/pterodactyl")' "$compose_file" | head -n 1)
    
    # If the path contains a variable, expand it
    if [[ $wings_path == \$* ]]; then
        wings_path=$(eval echo $wings_path)
    fi
    
    echo "$wings_path/config.yml"
}

# Load database variables
if [ -f /var/www/pterodactyl/.env ]; then
    source /var/www/pterodactyl/.env
    echo "Loaded .env file successfully."
elif [ -f ./docker-compose.yml ]; then
    extract_db_info_from_compose ./docker-compose.yml
    echo "Extracted database info from local docker-compose.yml"
elif [ -f /srv/pterodactyl/docker-compose.yml ]; then
    extract_db_info_from_compose /srv/pterodactyl/docker-compose.yml
    echo "Extracted database info from /srv/pterodactyl/docker-compose.yml"
else
    echo "Failed to find .env or docker-compose.yml file."
    while true; do
        echo "Would you like to:"
        echo "1) Manually specify a path to your Pterodactyl Panel's .env or docker-compose.yml file"
        echo "2) Enter the information to connect to your panel database manually"
        echo "3) Exit"
        read -p "Enter your choice (1/2/3): " choice

        case $choice in
            1)
                read -p "Enter the path to your .env or docker-compose.yml file: " config_path
                if [[ $config_path == *.env ]]; then
                    source "$config_path"
                    echo "Loaded .env file from $config_path successfully."
                elif [[ $config_path == *docker-compose.yml ]]; then
                    extract_db_info_from_compose "$config_path"
                    echo "Extracted database info from $config_path"
                else
                    echo "Invalid file type. Please specify an .env or docker-compose.yml file."
                    continue
                fi
                break
                ;;
            2)
                read -p "Enter DB_HOST: " DB_HOST
                read -p "Enter DB_PORT: " DB_PORT
                read -p "Enter DB_USERNAME: " DB_USERNAME
                read -p "Enter DB_DATABASE: " DB_DATABASE
                read -p "Enter DB_PASSWORD: " DB_PASSWORD
                echo "Database information entered manually."
                break
                ;;
            3)
                echo "Exiting."
                exit 1
                ;;
            *)
                echo "Invalid choice. Please enter 1, 2, or 3."
                ;;
        esac
    done
fi

# Get "data" path from config.yml file
if [ $DOCKER_DETECTED -eq 1 ]; then
    config_path=$(find_wings_config_path "$COMPOSE_FILE")
    echo "Wings config path: $config_path"
else
    config_path="/etc/pterodactyl/config.yml"
fi

if [ ! -f "$config_path" ]; then
    echo "Failed to find Wings config file at $config_path. Exiting."
    exit 1
fi

data_path=$(grep 'data:' "$config_path" | awk '{print $2}')
if [ -z "$data_path" ]; then
    echo "Failed to find data path in $config_path. Exiting."
    exit 1
fi

echo "Data path: $data_path"

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
                if sudo setfacl -Rm u:$REAL_USER:rx $CURRENT_PATH; then
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

# Function to execute MySQL query
execute_mysql_query() {
    local query="$1"
    if [ $DOCKER_DETECTED -eq 1 ]; then
        echo "Executing query in Docker environment..."
        docker compose -f "$COMPOSE_FILE" exec -T database sh -c 'mysql -u"$MYSQL_USER" -p"$MYSQL_PASSWORD" "$MYSQL_DATABASE" -e "'"$query"'"'
    elif command -v mysql &> /dev/null; then
        mysql -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USERNAME" -p"$DB_PASSWORD" "$DB_DATABASE" -e "$query"
    elif command -v mariadb &> /dev/null; then
        mariadb -h"$DB_HOST" -P"$DB_PORT" -u"$DB_USERNAME" -p"$DB_PASSWORD" "$DB_DATABASE" -e "$query"
    else
        echo "Error: Neither MySQL client nor Docker Compose setup found. Cannot execute database query."
        exit 1
    fi
}

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
echo "Executing database query..."
results=$(execute_mysql_query "$query")
if [ $? -ne 0 ]; then
    echo "Failed to execute database query. Please check your database connection."
    exit 1
fi
echo "Database query executed successfully."

# Count the frequency of each name
while read -r uuid name; do
    # Check if UUID directory exists and it's not the .sftp directory
    if [ -d "$data_path/$uuid" ] && [ "$uuid" != ".sftp" ]; then
        # Increment the frequency count
        name_freq["$name"]=$((${name_freq["$name"]-0}+1))
    fi
done <<< "$results"

# Iterate over results from the query again to create symlinks
while read -r uuid name; do
    # Check if UUID directory exists and it's not the .sftp directory
    if [ -d "$data_path/$uuid" ] && [ "$uuid" != ".sftp" ]; then
        # If name frequency is more than 1, append the count
        if [[ ${name_freq["$name"]} -gt 1 ]]; then
            # Initialize or increment the count
            name_count["$name"]=$((${name_count["$name"]-0}+1))
            # Append the count to the name
            current_name="${name}${name_count["$name"]}"
        else
            current_name="$name"
        fi

        # Check if user has rwx permissions through ACL
        if ! su - $REAL_USER -c "[ -r $data_path/$uuid ] && [ -x $data_path/$uuid ] && [ -w $data_path/$uuid ]"; then
            if [ $PERMISSION_FIX_AGREED -eq 1 ]; then
                sudo setfacl -m u:$REAL_USER:rwx $data_path/$uuid
                echo "Permissions fixed for $data_path/$uuid."
            else
                while true; do
                    echo "User $REAL_USER does not have read, write, and execute access to $data_path/$uuid. Would you like to add it? (y/n)"
                    read answer </dev/tty
                    if echo "$answer" | grep -iq "^y$"; then
                        sudo setfacl -m u:$REAL_USER:rwx $data_path/$uuid
                        echo "Permissions fixed for $data_path/$uuid."
                        PERMISSION_FIX_AGREED=1
                        break
                    elif echo "$answer" | grep -iq "^n$"; then
                        echo "No changes made to permissions."
                        break
                    else
                        echo "Invalid input. Please type 'y' or 'n'."
                    fi
                done
            fi
        fi

        # Create symlink(s) in the home directory
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
