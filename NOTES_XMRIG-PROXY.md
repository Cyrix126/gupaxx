# Integration of Xmrig-Proxy

## Objective

Allows a user to point his miners on the Gupaxx instance.

1/ to have the sum of the HR in his stats

2/ to let the algorithm of distribution of HR controls the HR of all the external miners.

## UI implementation

New Tab to start Xmrig-Proxy, interact with console output, give custom options, select a pool from the pool list.
New process column in Status Tab for Xmrig-Proxy.

## Technical implementation

xmrig-proxy will mine on p2pool instead of xmrig.
When xmrig-proxy is enabled, xmrig is automatically redirected to it instead of p2pool.
XvB algo will check if xmrig-proxy is enabled and watch/control his data instead of xmrig.
