# TODO

# Bounty

[XvB Bounty](https://bounties.monero.social/posts/105)

- [x] upgrade deps
- [ ] fix clippy
- [ ] better organize some new code
- [ ] merge commits from upstream
- [x] separate logic in smaller modules
- [x] new tab XvB
  - [x] logo
  - [x] link to website
  - [x] link and message hovering explaining registration and needs to read the rules.
  - [x] token input
  - [x] hero checkbox
  - [x] log section
    - [x] state of XvB process
    - [ ] algorithm decisions
  - [x] private stats
      - [x] round type in
      - [x] win or loose
      - [ ] fix: remove B symbol for HR
  - [x] new process for XvB
    - [x] status process XvB
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
- [ ] edit metadata of project
  - [ ] adapt README for XvB
  - [ ] adapt doc for new code
  - [ ] cargo package metadata
  - [ ] pgp signatures
