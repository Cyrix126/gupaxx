# TODO

# Bounty

[XvB Bounty](https://bounties.monero.social/posts/105)

- [x] upgrade deps
- [x] fix clippy
- [x] separate logic in smaller modules
- [x] new tab XvB
  - [x] logo
  - [x] link to website
  - [ ] message overing explaining registration and needs to read the rules.
  - [ ] token input
  - [ ] information
    - [ ] status of h/s received by the raffle, authenfication by token.
    - [ ] status of 1h and 24h average h/s sent to raffle from this instance
    - [ ] number of failures
    - [ ] log section
      - [ ] winner of round
      - [ ] round in
  - [ ] hero checkbox
- [x] status process XvB
  - [x] public information from [API](https://xmrvsbeast.com/p2pool/stats)
- [ ] if not enough hashrate for min round and share acquirement OR no share acquired, node destination for xmrig is only p2pool.
- [ ] if share acquired and enough hashrate to keep up round min hashrate and share acquirement, switch node destination for xmrig between p2pool and raffle giving raffle minimum round requirement + buffer.
- [ ] if hero checked, give maximum hasrate to raffle while keeping enough for p2pool.
- [ ] edit metadata of project
  - [ ] cargo package metadata
  - [ ] pgp signatures
