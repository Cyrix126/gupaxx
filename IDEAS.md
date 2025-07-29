# IDEAS for the future of Gupaxx

**Theses are only ideas, everything here is still to be decided and only thoughts for now.**
Some ideas could be done in a matter of hours, some could take months.

## More Decentralized
### Synchronize source code repository on p2p network
Github is proprietary. If Gupaxx aims to be free software, it should not only be available on this proprietary platform and we should explore other options to be github free.
We can use [Radicle](https://radicle.xyz/) to get Gupaxx on a p2p collaboration stack.
We can create mirrors between Github and Radicle.
The code, issues and PR could be synchronized with Github.
### Integrate a P2Pool compatible Nodes crawler
To get rid of integrating nodes list, we can include a crawler like monero.fail inside Gupaxx.
### Integrate a Monero Node
[Done](https://github.com/Cyrix126/releases/tag/v1.5.0)  
If we want Gupaxx to help user mine in the most decentralized way, we should offer them to run a monero node.
This would be optional and would check if the requirement are fulfilled before enabling the button to do so.

## More User friendly
### Website
Creating a website like [gupax.io](https://gupax.io) to have a more user friendly presentation and installation method.
Having a website, we can detect the architecture and OS of the visitor and give them the right archive to download.
### XMRvsBeast explanation
Currently in the [README](https://github.com/Cyrix126/gupaxx#what-is-gupaxxxmrvsbeast), there is no explanation on what XMRvsBeast is.
The README explains what XMRvsBeast does but not what it is.
we could improve this with either creating another chapter or expanding the [current one](https://github.com/Cyrix126/gupaxx#what-is-gupaxxxmrvsbeast).
### Generated wallet
If Gupaxx could create a wallet and put the primary address in P2Pool tab automatically, it would remove a manual step for the user.
It could be an option to ask at first start.
The user could access this wallet on the same computer with the official GUI wallet. A password would be needed and a button could be added to show the seedphrase.
### Auto register to XvB
If Gupaxx could register the user automatically to the raffle, it would remove a manual step for the user.
Automatic registration could be done to prevent spam by checking if the registered address is in P2Pool window or by giving a small HR to XvB.
It could be an option to ask at first start.
### Setup Guide
At first start, a guide could ask the user what it intends to do with Gupaxx (create node, create wallet, use XMRig-Proxy, participate in raffle...) and do the setup for him and show him what it must do manually. An option to skip this guide would be present for advanced users.
### Very noob mode
At first start, a mode is suggested for very noob users. It would only shows the seed phrase of the wallet generated and configure all options for the user.
### Use remote node while syncing local node
[Done](https://github.com/Cyrix126/gupaxx/commit/075beddea19b3f09e1e7b2327e235814fe588520)  
To reduce the time to get the first shares. No need to wait for the monero node to be synced.

### Better UI
#### Do not re-ask password if not needed
On Linux, Do not re-ask for sudo to start XMRig when the user can use sudo without a password. It can happen if visudo has been configured to do so or if there is a delay specified in /etc/sudoers with "timestamp_timeout".
#### Scrolling arrows
To notify the user that content is present in the bottom, an arrow pointing downside will appear.
### Warning about new update available
If the user disabled auto updates, show a message when a new update is available with the changelog. Allows user to dismiss the update.
### Better defaults
[Done](https://github.com/Cyrix126/gupaxx/commit/6cb767a342bec2df3358b10826a1ec1dee57fc76) and [Done](https://github.com/Cyrix126/gupaxx/commit/a102bdbee2e4c0bc8785f9e638d3e54958d79489)  
Reduce in/out peers, remote nodes by default
#### Set fixed font size, do not resize with size of window
[Done](https://github.com/Cyrix126/gupaxx/releases/tag/v1.6.0)  
Setting a fixed font size will allow to use the space fully and having a UI more adapted to screens. Option to set the size of the font will be included.
#### Allow resize of consoles
[Done](https://github.com/Cyrix126/gupaxx/releases/tag/v1.7.0)  
So users can view more or less output as they need.
#### Friendlier custom args
[Done](https://github.com/Cyrix126/gupaxx/releases/tag/v1.7.0)  
For custom command arguments, some args are required. To help the user not make any errors, theses args must be prefilled. The user will need to enable a checkbox to apply the custom command arguments. A button reset will replace the text fields by only the required fields.
#### Allow to hide status column
[Done](https://github.com/Cyrix126/gupaxx/releases/tag/v1.7.0)  
Status columns can take together lots of space and user can use only a number of them. Allows to hide/restore them with button on the bottom on the columns for each one.

## Making Gupaxx Support more environments
### Packaging
Add repository/AUR for Gupaxx and a status of packaging distro/version on the README.
Add support support for [Flatpak](https://docs.flatpak.org/en/latest/first-build.html).
Add support for [AppImage](https://appimage.org/).
Add support for [Nix](https://nixos.wiki/wiki/Nixpkgs/Create_and_debug_packages).
Add support for [Guix](https://guix.gnu.org/).
Add support for [*BSD](https://www.freebsd.org/) systems.
Add support for [Scoop](https://scoop.sh/).
Add support for [Docker](https://docs.docker.com/).
Add support for [DEB](https://wiki.debian.org/Packaging).
### Minimum requirement
Add a table with the minimum hardware and software requirements to the README.
### Add more target
Gupaxx could add support for Linux ARM64 since both P2Pool and XMRig can compile on this target.
### CLI for Algorithm
A simple script or a small binary could be made to reproduce the algorithm who would take args for every other needed programs.
This script would need arguments to know how to control XMRig/XMRig-Proxy and where to watch P2Pool data plus the XvB token and XMR address.
### Web UI
To be able to control and watch Gupaxx from another device, a daemon mode could be built with a web UI front-end.
### Refactor size of text
[Done](https://github.com/Cyrix126/gupaxx/releases/tag/v1.6.0)  
Gupaxx currently resize texts/widgets based on the window size. Instead, the text/widget size should be decided by the OS/config, scroll bar should be used when there is not enough space. It will allow to use Gupaxx on different ratio of screen.

## More Powerful
### Optimization for XMRig
#### Add automatic options
On Linux, we can activate 1GB pages after detecting CPU flags. We can also add cpu affinity option.
#### Manual optimizations
On the XMRig tab, inform users about manual optimizations that Gupaxx can't control. For example, disabling hyper-threading in BIOS is recommended.
### Automatic sending of funds
A way to automatically send funds of mining to a wallet address or multiple wallet addresses by setting a minimum amount and % with time frequency or setting a fixed amount and priority.
### Wait for sync to start of XMRig
If P2Pool/node is not yet synced, XMRig can slower them and mine for nothing if it start at the same time. We don't want to prevent the user to start XMRig without P2Pool, so XMRig could start later only if P2Pool is auto started.
### Systray icon
Enable a way to put Gupaxx in background, managing it with a systray icon.
### Auto-Launch
Option to launch Gupaxx at startup
### API of Algorithm
To make the Algorithm controllable outside of Gupaxx.
### Graphs history of HR
The user could see how the HR was given on P2Pool and XvB.
### Update XMRig benchmark from Gupaxx
To have the latest benchmark from XMRig, but still including one by default. Also automates the inclusion on release.
### Auto restart after updates
Updates can be applied only when Gupaxx is restarted. Make a button to auto-restart after updates.
### Ban spy node list recommended by MRL
https://github.com/monero-project/meta/issues/1124
Enabled by default, button to disable.
### New suit of tests, including testing interaction of widgets
A lot of tests since the fork makes less sense and a lot of new situation needs to be tested. Time should be taken to add new tests and make Gupaxx more robust.
egui_kittest library can be used to test the interaction of the UI directly. 
### Manually set HR for XvB algo
Done by [Sina](https://github.com/mostafaei2002) [PR](https://github.com/Cyrix126/gupaxx/pull/11)  
An advanced tab on XvB tab with multiple tools to set the HR manually.
The user can sometime better know the right decision from his HR than the algo that will take more time to get everything right, specially if resources are changing.
### Integrate XMRig-Proxy
[Done](https://github.com/Cyrix126/gupaxx/releases/tag/v1.2.0)  
The algorithm of distribution of HR can't control HR outside of his instance.
It must estimate external HR, which can be approximative.
If a user control multiples miners, it could connect all of them to a XMRig-Proxy instance.
Gupaxx could offer this XMRig-instance and control it like it was a normal XMRig instance.
### Watch Stratum Data instead of estimate.
[Done](https://github.com/Cyrix126/gupaxx/releases/tag/v1.7.0)  
Right now, the algorithm estimate the eHR with the estimation made by the P2Pool instance which is calculating from passed shares.
The algorithm could instead watch the stats from the stratum server, which is more precise but would take into account only miners which are pointed to it.
The algorithm would still check the estimation made by the P2Pool instance of Gupaxx and warn the user if it seems there is too much difference between the data of the stratum server and the one of P2Pool. It could prevent the user to forget to configure a miner to the stratum P2Pool.
Could also be an option in advanced tab of XvB warning the user that he should point all his miners to the P2Pool instance of Gupaxx to take them into account.
It can be a checkbox into advanced option of XvB to use the stratum data.

## Privacy
### Buttons enabling Tor/I2P
Allow to torify every processes. For mining, it can add a big disadvantage because of the latency. More research on the impact on missed rewards are needed to be able to warn the user of how much gaines he could miss.
### Description of data given to servers
Describing what data can be saved/collected/published and the privacy impact while interacting with:
- updates
- xmrig benchmarks
- remote nodes
- monero network
- p2pool network
- XvB Raffle

## Trust-less Builds
### Reproducible builds
To remove (un)necessary trust, binaries released should have the same checksum if recompiled without code change.
See [This](https://reproducible-builds.org).
### Bootstrapable Builds
To remove (un)necessary trust, binaries released should have the same checksum if recompiled without code change.
This ensures that the build process is transparent and verifiable, enhancing the security and integrity of the software.
See [This](https://bootstrappable.org).
### Release changes notes preview
Show the summuray of what will change between releases before updating to newer release.
### Check signature of updates with Gupaxx
let the build in updater of Gupaxx check the signature of the release to confirm that the releases has been signed by the right key.

## Donation
### Donation transparency
[Done](https://kuno.anne.media/fundraiser/dsrr)
So that user can see how much is given to this project and make their own opinion of it if enough donations have been given or not, the history of donation should be made visible with the viewkey available.
#### Kuno page
[Done](https://kuno.anne.media/fundraiser/dsrr)
A page describing the time required for each release, keeping track of funds per releases. Informing donators of the ROADMAP and what they can expect from donating.  

## XvB
keeping track of participation history in rounds and automatically showing results in the XvB tab.
