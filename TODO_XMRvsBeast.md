# TODO

# Bounty

[XvB Bounty](https://bounties.monero.social/posts/105)

*These todos are not all part of the bounty.*

- [x] upgrade deps
- [x] separate logic in smaller modules
- [x] new tab XvB
  - [x] logo
  - [x] link to website
  - [x] link and message hovering explaining registration and needs to read the rules.
  - [x] token input
  - [x] hero checkbox
  - [x] log section
    - [x] state of XvB process
    - [x] selected XvB node
    - [x] algorithm decisions info
    - [x] timestamp
  - [x] private stats
      - [x] from XvB API (fails, average 1h and 24h)
      - [x] round type in
      - [x] win or loose
  - [x] new process for XvB
    - [x] update preferred XvB node based on ping and backup 
      - [x] fix: xmrig will not do anything if node is not responding. Need to parse output of xmrig for error and update nodes.
    - [x] status process XvB
    - [x] status process XMRig node in real time.
    - [x] public information from [API](https://xmrvsbeast.com/p2pool/stats)
    - [x] stop, start, restart buttons
    - [x] button to autostart
    - [x] distribute hashrate conforming to the algorithm.
      - [x] check every 10 minutes average Xmrig HR of last 15 minutes
      - [x] ask Xmrig to mine on p2pool
        - [x] generate token for xmrig
        - [x] enable xmrig with remote access control
      - [x] check if at least a share in pplns Window
      - [x] calculate spared HR
      - [x] calculate time to be spared
        - [x] with hero option
        - [x] without hero option, to give minimum to be in most accessible round type
      - [x] sleep 10mn less time to spare then ask Xmrig to mine on XvB node
    - [x] output log to console in XvB tab
- [x] fix some overlapping from upstream
- [ ] edit metadata of project
  - [ ] adapt README for XvB 
    - [ ] beta release
    - [ ] stable release
  - [ ] video tutorial to set up XvB Tab
  - [ ] adapt doc for new code
  - [ ] cargo package metadata
  - [ ] pgp signatures
- [x] fix clippy
- [ ] optimizations
  - [x] benchmarks table render only what is visible
  - [x] console output render only what is visible
  - [ ] migrate to hyper stable
  - [ ] use tor socks proxy instead of creating one
- [ ] better organize new code
- [x] merge commits from upstream
- [ ] tests for new functions
- [ ] pre-release
  - [ ] feedback
- [ ] release
