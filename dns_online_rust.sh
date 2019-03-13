#!/usr/bin/env sh

export ONLINE_API_KEY="<YOUR API KEY HERE>"

#Usage: dns_myapi_add _acme-challenge.www.domain.com  "XKrxpRBosdIKFzxW_CT3KLZNf6q0HG9i01zxXp5CPBs"
dns_online_rust_add() {
    dnsapi/le_dns_online add_record LE-challenge-$(date +%s) "$ONLINE_API_KEY" $1 $2
}

#Usage: dns_myapi_rm _acme-challenge.www.domain.com  "XKrxpRBosdIKFzxW_CT3KLZNf6q0HG9i01zxXp5CPBs"
dns_online_rust_rm() {
    dnsapi/le_dns_online delete_record LE-challenge-done-$(date +%s) "$ONLINE_API_KEY" $1 $2
}
