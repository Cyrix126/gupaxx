# Notes CLI


## Features
  
- fetch p2pool node stratum data
- start XMRig instance.
- stop with descriptive errors if p2pool/xmrig have issue at launch.
- output status of algo
- output on demand public stats
- output on demand account stats

## Launch args
- XVB token
- XMR address
- p2pool address:port
- optional: hero
- optional: xmrig custom additional args 
- optional: quiet algo
- optional: quiet xmrig
- optional: path of xmrig

Example:

```
gupaxx --cli --token xxxxx --address xxxxx --hero --p2pool="127.0.0.1:3333" --xmrig-add-args="--xxx --xxx" -t 8 -q --path-xmrig="/path/to/xmrig-binary"
```

## Commands
Possible input at runtime:  
- all commands of xmrig: transfer the commands to the xmrig instance and return output.
- pubstats/ps: returns the stats of the public API.
- accountstats/as: returns the stats of your account.
- quit: quit the program, shutting down xmrig.
Example

```
as ↵
failures: 0
donated_last_hour: 0.00kH/s
donated_last_24_hours: 0.00kH/s
Round: VIP
You are not the winner
```


## Technical implementation

The cli args are managed by [clap](https://docs.rs/clap).
The code for managing current args from upstream will be replaced to use this crate.
