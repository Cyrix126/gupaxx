# Integration of Xmrig-Proxy

## Objective

Allows a user to point his miners on the Gupaxx instance.

1/ to have the sum of the HR in his stats

2/ to let the algorithm of distribution of HR controls the HR of all the external miners.

## UI implementation

It is not useful to someone who have only one miner, so it needs to be manually enabled.


Tab to start Xmrig-Proxy, interact with console output and give custom options.
Message on Xmrig tab indicating xmrig is automatically redirected to it.
Local ip will be displayed.
Button to fetch public ip to know at which one to point miner at.

Status column of xmrig is replaced by xmrig-proxy when it is enabled.
Xmrig-proxy will display stats for each miners.

## Technical implementation

xmrig-proxy will mine on p2pool instead of xmrig.
When xmrig-proxy is enabled, xmrig is automatically redirected to it instead of p2pool.
XvB algo will check if xmrig-proxy is enabled and watch/control his data instead.

## TODO

- [x] State
- [x] Helper thread
- [x] impl Helper
- [x] UI
- [x] interaction with xmrig
- [ ] advanced tls, keep alive, ip port
- [ ] info about ip and firewall
- [ ] status tab
- [ ] interaction with xvb
