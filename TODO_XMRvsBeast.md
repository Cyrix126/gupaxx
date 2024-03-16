# TODO

# Bounty

[XvB Bounty](https://bounties.monero.social/posts/105)

- [x] upgrade deps
- [x] fix clippy
- [x] separate logic in smaller modules
- [x] new tab XvB
  - [x] logo
  - [x] link to website
  - [x] link and message hovering explaining registration and needs to read the rules.
  - [x] token input
  - [x] hero checkbox
  - [x] log section
    - [x] state of XvB process
  - [x] private stats
      - [x] round type in
      - [x] win or loose
  - [ ] new process for XvB
    - [x] status process XvB
    - [x] public information from [API](https://xmrvsbeast.com/p2pool/stats)
    - [x] stop, start, restart buttons
    - [x] button to autostart
    - [ ] distribute hashrate conforming to the algorithm.
      - [x] check every 10 minutes average Xmrig HR of last 15 minutes
      - [ ] ask Xmrig to mine on p2pool
        - [ ] generate token for xmrig
        - [ ] enable xmrig with remote access control
      - [x] check if at least a share in pplns Window
      - [ ] calculate spared HR
      - [ ] calculate time to be spared
        - [ ] with hero option
        - [ ] without hero option, to give minimum to be in most accessible round type
      - [ ] sleep 10mn less time to spare then ask Xmrig to mine on XvB node
    - [x] output log to console in XvB tab
- [ ] edit metadata of project
  - [ ] cargo package metadata
  - [ ] pgp signatures
