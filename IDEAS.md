# **IDEAS for the future of Gupaxx**

**Theses are only ideas, everything here is still to be decided and only thoughts for now.**
Some ideas could be done in a matter of hours, some could take months.

## **More Decentralized**
### Synchronize source code repository on p2p network
Github is proprietary. If Gupaxx aims to be free software, it should not be only available on this proprietary platform and we should explore options to get github free.  
We can use [Radicle](https://radicle.xyz/) to get Gupaxx on a p2p collaboration stack.  
The code, issues and PR could be synchronized with Github.  
### Integrate a P2Pool compatible Nodes crawler
To get rid of integrating nodes list, we can include a crawler like monero.fail inside Gupaxx.
### Integrate a Monero Node
[Done](https://github.com/Cyrix126/releases/tag/v1.5.0)
If we want Gupaxx to help user mine in the most decentralized way, we should offer them to run a monero node.
This would be optional and would check if the requirement are fulfilled before enabling the button to do so.

## **More User friendly**
### Website
Build a website like [gupax.io](https://gupax.io) to have a more user frendly presentation and installation method.  
Having a website, we can detect the architecture and os of the visitor and give him the right archive to download.
### Generated wallet
If Gupaxx could create a wallet and put the primary address in p2pool tab automaticcly, it would remove a manual step for the user.  
It could be an option to ask at first start.  
The user could access this wallet on the same computer with the official GUI wallet. A password would be needed and a button could be added to show the seedphrase.
### Auto register to XvB
If Gupaxx could register the user automaticcly to the raffle, it would remove a manual step for the user.  
Automatic registration could be done to prevent spam by checking if the registered address is in p2pool window or by giving a small HR to XvB.
It could be an option to ask at first start.  
### Setup Guide
At first start, a guide could ask the user what it intends to do with Gupaxx (create node, create wallet, use xmrig-proxy, participate in raffle...) and do the setup for him and show him what it must do manually. An option to skip this guide would be present for advanced users.
### Very noob mode
At first start, a mode is suggested for very noob users. It would only shows the seed phrase of the wallet generated and configure all options for the user.
### Use remote node while syncing local node
To reduce the time to get the first shares. No need to wait for the monero node to be synced.

### **Better UI**
#### Set fixed font size, do not resize with size of window
Setting a fixed font size will allow to use the space fully and having a UI more adapted to screens. Option to set the size of the font will be included.
#### Do not re-ask password if not needed
On Linux, Do not re-ask for sudo to start xmrig when the user can use sudo without a password. It can happen if visudo has been configured to do so or if there is a delay specified in /etc/sudoers with "timestamp_timeout". 
#### Allow resize of consoles
So users can view more or less output as they need.
#### Scrolling arrows
To notify the user that content is present in the bottom, an arrow pointing downside will appear.
#### Friendlier custom args
For custom command arguments, some args are required. To help the user not make any errors, theses args must be prefilled. The user will need to enable a checkbox to apply the custom command arguments. A button reset will replace the text fields by only the required fields.
#### Allow to hide status column
Status columns can take together lots of space and user can use only a number of them. Allows to hide/restore them with button on the bottom on the columns for each one.
#### Better default
[Done](https://github.com/Cyrix126/gupaxx/commit/6cb767a342bec2df3358b10826a1ec1dee57fc76) and [Done](https://github.com/Cyrix126/gupaxx/commit/a102bdbee2e4c0bc8785f9e638d3e54958d79489)
Reduce in/out peers, remote nodes by default

## **Making Gupaxx Support more environments**
### Packaging
Add repository/AUR for Gupaxx and a status of packaging distro/version on the README
### Minimum requirement
Add on README a table with minimum hardware/software requirements.
### Add more target
Gupaxx could add support for linux arm64 since p2pool and xmrig can compile on this target. 
### Refactor size of text
Gupax/x currently resize texts/widgets based on the window size. Instead, the text/widget size should be decided by the OS/config, scroll bar should be used when there is not enough space. It will allow to use Gupaxx on different ratio of screen.
### CLI for Algorithm
A simple script or a small binary could be made to reproduce the algorithm who would take args for every other needed programs.
This script would need arguments to know how to control xmrig/xmrig-proxy and where to watch p2pool data plus the XvB token and xmr address.
### Web UI
To be able to control and watch Gupaxx from another device, a daemon mode could be built with a web UI front-end.

## **More Powerful**
### Optimization for xmrig
#### Add automatic options
On linux, we can activate 1GB pages after detecting cpu flags. We can also add cpu affinity option.
#### Manual optimizations
On the XMRig tab, inform users about manual optimizations that Gupaxx can't control. For example, disabling hyper-threading in BIOS is recommended.
### Watch Stratum Data instead of estimate.
Right now, the algorithm estimate the eHR with the estimation made by the p2pool instance which is calculating from passed shares.
The algorithm could instead watch the stats from the stratum server, which is more precise but would take into account only miners which are pointed to it.
The algorithm would still check the estimation made by the p2pool instance of Gupaxx and warn the user if it seems there is too much difference between the data of the stratum server and the one of p2pool. It could prevent the user to forget to configure a miner to the stratum p2pool.
Could also be an option in advanced tab of XvB warning the user that he should point all his miners to the p2pool instance of Gupaxx to take them into account.
It can be a checkbox into advanced option of XvB to use the stratum data.
### Automatic sending of funds
A way to automatically send funds of mining to a wallet address or multiple wallet addresses by setting a minimum amount and % with time frequency or setting a fixed amount and priority.
### Wait for sync to start of xmrig
If p2pool/node is not yet synced, xmrig can slower them and mine for nothing if it start at the same time. We don't want to prevent the user to start xmrig without p2pool, so xmrig could start later only if p2pool is auto started.
### Systray icon
Enable a way to put Gupaxx in background, managing it with a systray icon.
### Auto-Launch
Option to launch Gupaxx at startup 
### API of Algorithm
To make the Algorithm controllable outside of Gupaxx.
### Graphs history of HR
The user could see how the HR was given on p2pool and XvB.
### Update xmrig benchmark from Gupaxx
To have the latest benchmark from xmrig, but still including one by default. Also automates the inclusion on release
### Manually set HR for XvB algo
Done by [Sina](https://github.com/mostafaei2002) [PR](https://github.com/Cyrix126/gupaxx/pull/11)
An advanced tab on XvB tab with multiple tools to set the HR manually.
The user can sometime better know the right decision from his HR than the algo that will take more time to get everything right, specially if resources are changing.
### Integrate XMRig-Proxy
[Done](https://github.com/Cyrix126/gupaxx/releases/tag/v1.2.0)  
The algorithm of distribution of HR can't control HR outside of his instance.
It must estimate external HR, which can be approximative.
If a user control multiples miners, it could connect all of them to a xmrig-proxy instance.
Gupaxx could offer this xmrig-instance and control it like it was a normal xmrig instance.  

## **Trust-less Builds**
### Reproducible builds
To remove necessary trust, binairies released should have the same checksum if recompiled without code change.
See [this](https://reproducible-builds.org).
### Release changes notes preview
Show the summuray of what will change between releases before updating to newer release.
### Check signature of updates with Gupaxx
let the build in updater of Gupaxx check the signature of the release to confirm that the releases has been signed by the right key.

## **Donation**
### Donation transparency
So that user can see how much is given to this project and make their own opinion of it if enough donations have been given or not, the history of donation should be made visible with the viewkey available.  

## **XvB**
keeping track of participation history in rounds and automatically showing results in the XvB tab.
