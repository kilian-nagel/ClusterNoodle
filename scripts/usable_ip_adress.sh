#!/bin/bash

get_all_interfaces_names() {
    ip -o link show | awk -F': ' '{print $2}'
}

# Ne récupère les interfaces de type bridge et localhost.
filter_usable_interfaces(){
    readarray -t interfaces < <(get_all_interfaces_names)
    usable_interfaces=()
    i=0
    for interface in "${interfaces[@]}"
    do
        if ! [[ "$interface" == *docker* ]] && ! [[ "$interface" == *br-* ]] && ! [[ "$interface" == *vnet* ]] && ! [[ "$interface" == *lo* ]]
        then
            usable_interfaces[$i]=$interface
            ((i++))
        fi
    done
    printf '%s\n' "${usable_interfaces[@]}"
}

# Récupère une seule interface réseau en fonction de priorité : ethernet sinon si ça n'existe, le wi-fi etc...
get_usable_interface() {
    mapfile -t usable_interfaces < <(filter_usable_interfaces | sort -u)
    
    local priorities=("eth" "wlp" "virbr")
    for prefix in "${priorities[@]}"; do
        for iface in "${usable_interfaces[@]}"; do
            if [[ "$iface" == "$prefix"* ]]; then
                echo "$iface"
                return 0
            fi
        done
    done

    if (( ${#usable_interfaces[@]} > 0 )); then
        echo "${usable_interfaces[0]}"
        return 0
    fi
    return 1
}


get_usable_interface