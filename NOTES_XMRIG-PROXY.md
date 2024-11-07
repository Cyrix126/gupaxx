# **Integration of Xmrig-Proxy**

## **Objective**

Allows a user to point his miners on the Gupaxx instance.

    1 to have the sum of the HR in his stats

    2 to let the algorithm of distribution of HR controls the HR of all the external miners.

## **UI implementation**

New Tab to start XMRig-Proxy, interact with console output, give custom options, select a pool from the pool list.

New process column in Status Tab for XMRig-Proxy.

## **Technical implementation**

XMRig-Proxy will mine on P2Pool instead of XMRig.
When XMRig-proxy is enabled, XMRig is automatically redirected to it instead of P2Pool.

XvB algo will check if XMRig-Proxy is enabled and watch/control his data instead of XMRig.
