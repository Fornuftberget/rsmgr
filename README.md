# rsmgr

**rsmgr** is a tool used by Förnuftberget RolePlay to update resources on the server and development environments.

***

### Usage
```
Förnuftberget Resource Manager

USAGE:
    rsmgr [FLAGS] [OPTIONS]

FLAGS:
        --discard    Discard any local changes
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose output

OPTIONS:
    -b, --base_path <base_path>        Base path to resources [default: resources/]
    -u, --base_url <base_url>          Git base url [default: git@github.com:Fornuftberget/]
    -c, --config <config>              Path to server.cfg [default: server.cfg]
    -n, --num_workers <num_workers>    How many workers(threads) to use for resource updates
    -k, --ssh_key <ssh_key>            Path to ssh-key, if dir then we will look for a key, if omitted then we will try
                                       to use ssh-agent
```

Example server.cfg
```
# The format is <repository name>:<clone destination>

#git_clone script1:[fbrp]/script1
#git_clone script2:[misc]/script2
#git_clone interiors:[interiors]
```

***

### Building

rsmgr is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.

To build rsmgr:

```
$ git clone https://github.com/Fornuftberget/rsmgr
$ cd rsmgr
$ cargo build --release
$ ./target/release/rsmgr --version
rsmgr 0.1.0
```
