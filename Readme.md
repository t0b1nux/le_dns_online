# le_dns_online

## What is it ?

le_dns_online is a frontend intended to provide dns validation of *Let's Encrypt* for the french cloud provider & hoster [online.net](https://www.online.net/en). More specifically, its goal is to integrate easily with [acme.sh](https://github.com/Neilpang/acme.sh).

## Building instructions

You need to use the Rust nightly compiler for now as I use the try_trait feature.
```
cargo build --release
```
And the binary is located in 'target/release/le_dns_online'.

## How do I install it ?

First, build the binary according to the 'building instructions' section.
You just need to add 'dns_online.sh' and the binary le_dns_online to the dnsapi folder inside '~/.acme.sh' (or whichever folder you use for acme.sh). You must then update the api_key in dns_online.sh to your private key (given at https://console.online.net/en/api/access) and you're good to go !

## How does it work ?

Acme.sh calls the fonction 'dns_online_add' from 'dns_online.sh', which calls le_dns_online binary.

le_dns_online then:

1) Add the record needed for *Let's Encrypt* in the current zone

2) Return its id to delete the record later on

Acme.sh takes back control again, and execute the authentification request. Subsequently, it calls 'dns_online_rm', which calls (again) le_dns_online binary.

This time, le_dns_online simply :

1) delete the temporary record with its id

And voilà ! You have your certs validated ;)

## Remaining work

Logging !

## Known issues

Do NOT use this program concurrently !!!
This may break the ongoing validation (or slightly worse than corrupting a free and simple process, corrupt your DNS zone, event if it's also less likely).

## Can I contribute ?

Sure, go ahead ! Prepare youself to dig your way through some terrible Rust code, however ^_^
