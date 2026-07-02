#!/bin/bash
loadConfig() {
    if [ ! -f "$1" ]; then
        echo "Config file not found: $1"
        exit 1
    fi
    source "$1"
}
para() {
    echo "backup_minecommit.sh [-i] [-h] [-r commit_id] [-l] [-f config_file]"
    echo "Description:"
    echo " -i    initial repo"
    echo " -h    show this help"
    echo " -r    restore to a specific commit"
    echo " -l    show commit log"
    echo " -f    read config file"
    exit 1
}

init() {
    read -p "Git repo folder name : " repo_name
    read -p "Minecraft map/world folder path (full path) : " map_loc

    if [ -z "$repo_name" ] || [ -z "$map_loc" ]; then
        echo "repo name and map path cannot be empty"
        exit 1
    fi

    if [ ! -d "$map_loc" ]; then
        echo "Map folder does not exist: $map_loc"
        exit 1
    fi

    repo_loc="$(pwd)/$repo_name"

    {
        echo "repo_loc='$repo_loc'"
        echo "map_loc='$map_loc'"
    } > "$(pwd)/$repo_name.conf"

    mkdir -p "$repo_loc" && echo "git repo folder created"
    git init --initial-branch main --bare "$repo_loc"
    git --git-dir "$repo_loc" config gc.auto 0
    git --git-dir "$repo_loc" config core.logAllRefUpdates true

    minecommit commit "$map_loc" "$repo_loc" --branch main --init --message "Auto commit: $(date '+%Y-%m-%d %H:%M:%S')" --repack
}

restore() {
    local commit_id="$1"
    minecommit checkout "$map_loc" "$repo_loc" --commit "$commit_id"
    exit 0
}

showLog() {
    git --git-dir="$repo_loc" log --oneline
    exit 0
}

while getopts 'ihr:l:f:' OPT; do
    case $OPT in
        i) init ;;
        h) para ;;
        r)
            if [ -z "$repo_loc" ] || [ -z "$map_loc" ]; then
                echo "No config found, please run -i first"
                exit 1
            fi
            restore "$OPTARG"
            ;;
        l)
            if [ -z "$repo_loc" ]; then
                echo "No config found, please run -i first"
                exit 1
            fi
            showLog
            ;;
        f) loadConfig "$OPTARG" ;;
        \?) echo "unknown option: -$OPTARG"; exit 1 ;;
        :) echo "option -$OPTARG requires an argument"; exit 1 ;;
        *) echo "unknown error while processing options"; exit 1 ;;
    esac
done
if [ -z "$repo_loc" ] || [ -z "$map_loc" ]; then
    echo "No config found, check -h for help"
    exit 1
fi

while true; do
    read -t 600 -p "commit message : " msg
    if [ -z "$msg" ]; then
        echo "No input, use timestamp"
        minecommit commit "$map_loc" "$repo_loc" --branch main --message "Auto commit: $(date '+%Y-%m-%d %H:%M:%S')" --repack
    elif [ "$msg" = "exit" ]; then
        echo "Exit"
        exit 0
    else
        minecommit commit "$map_loc" "$repo_loc" --branch main --message "Manual commit: $(date '+%Y-%m-%d %H:%M:%S') $msg" --repack
    fi
    echo "Complete, waiting for input message"
done
