![Gupaxx logo](assets/images/banner.png)

## Development Status
This fork has a stable release. 
It is intended for end users and offers a friendly and easy user experience.

## Gupaxx
`Gupaxx` is a fork of [**Gupax**](https://github.com/hinto-janai/gupax) integrating the [XMRvsBeast Raffle](https://xmrvsbeast.com), it is also a maintained software. Designed to simplify mining on [P2Pool](https://www.getmonero.org/2021/10/05/p2pool-released.html) while optionally participating (but you will want to 😉) in the XMRvsBeast raffle. 

## System requirements
`Gupaxx` may not run on machines with:
- A deprecated OS (Windows 7, Ubuntu 18.04, etc)
- CPU whithout support for OpenGL 3.1 (<2010)

[![CI](https://github.com/cyrix126/gupaxx/actions/workflows/ci.yml/badge.svg)](https://github.com/cyrix126/gupaxx/actions/workflows/ci.yml)

## Contents
* [What is Gupaxx/XMRvsBeast?](#what-is-gupaxxxmrvsbeast)  
* [Guide](#guide) 
* [XvB Tab](#xvb-tab) 
	- [Console](#console-of-xvb) 
	- [Token Input](#token-input) 
	- [Account stats](#account-stats) 
* [XvB Raffle](#xvb-raffle-status) 
* [Other changes](#other-changes) 
* [License](#license) 

## What is Gupaxx/XMRvsBeast?
[**`Gupaxx`**](https://getmonero.org) is a fork of [*Gupax*](https://github.com/hinto-janai/gupax) that integrates the [XMRvsBeast raffle](https://xmrvsbeast.com).

With this fork, you can easily split your hashrate between P2Pool and XMRvsBeast, increasing your chances of winning in the raffle while also supporting the Monero network via decentralizing the mining using using p2pool.

For a detailed explanation of Gupax, see the [README](https://github.com/hinto-janai/gupax) of upstream.


## Guide
1. [Download the bundled version of Gupaxx](https://github.com/Cyrix126/gupaxx/releases)
2. Extract
3. Launch Gupaxx

Next steps can be seen in this video tutorial:

https://github.com/Cyrix126/gupaxx/assets/58007246/610cbfea-fd97-4150-95ed-9c8a7ef5ba94



4. Input your Monero address in the `P2Pool` tab
5. Register the same address on [XMRvsBeast](https://xmrvsbeast.com)
6. Input the token received in the `XvB` Tab
6. Start `P2Pool`
7. Start `XMRig`
8. Start `XvB`

Gupaxx will distribute your hashrate between P2Pool and XMRvsBeast as defined by [this algorithm](NOTES_ALGORITHM.md).

The algorithm will decide which quantity of HR that will be directed to P2Pool and to XMRvsBeast, so that you still keep a share in the [PPLNS Window](https://github.com/SChernykh/p2pool#how-payouts-work-in-p2pool). 
It will by default send just enough to get to the highest round or, if hero mode is enabled, everything minus the minimum required to still have a share in the PPLNS Window.
</div>

## XvB Tab
![CI](assets/images/xvb_tab.png)
### Console of XvB
The output of the console will show useful information on the status of the XvB process and the decision of the algorithm for every 10 minutes.
### Token input
When you registered your XMR payout address, you should have received a token. Please enter this token here.
### Account stats
Account stats about your address on XMRvsBeast can be found here after the process is started with your token provided.


## XvB Raffle Status
Gupaxx adds a new column called **XvB Raffle** on the Status Tab in the Process submenu. It displays public statistics of XMRvsBeast, which are available [here](https://xmrvsbeast.com/p2pool).  
It is refreshed every minute.
This column will be active if the XvB process is started even partially, it doesn't need the token to be provided.

![XvB raffle stats](assets/images/xvb_raffle_stats.png)


## Other changes
This fork brings upgrades of dependence and some bugfixes about visual, performance and security that you can find in [DIFFERENCES](DIFFERENCES.md).  
~~I will eventually (meaning when I'll have time) create pull requests for upstream about these differences.~~  
**Edit**:  
There is currently no plan to upstream the changes as the owner of Gupax said he won't have time to review the PR.


## Troubleshooting
If you have any issue, feel free to ask for support in the [xmrvsbeast matrix room](#xmrvsbeast:monero.social) [![Chat on Matrix](https://matrix.to/img/matrix-badge.svg)](https://matrix.to/#/#xmrvsbeast:monero.social) or you can also just [open an issue](https://github.com/Cyrix126/gupaxx/issues/new/choose) in this repo. You can also contact me through [email](mailto:gupaxx@baermail.fr).
### Windows
You must add an exception to your antivirus for the directory where Gupaxx is executed. Follow the step for Windows Only, that starts at 30 seconds in this [video](https://user-images.githubusercontent.com/101352116/207978455-6ffdc0cc-204c-4594-9a2f-e10c505745bc.mp4).
### Mac OSX
You must remove Gupaxx app from quarantine with following command:  
*If you have put Gupaxx.app in your Applications*  
`xattr -d com.apple.quarantine /Applications/Gupaxx.app`
See this [issue](https://github.com/hinto-janai/gupax/issues/51).


## License

![GPL v3](assets/images/gplv3-with-text-136x68.png)

[Gupaxx](https://github.com/cyrix126/gupax/blob/master/LICENSE), [P2Pool](https://github.com/SChernykh/p2pool/blob/master/LICENSE), [XMRig](https://github.com/xmrig/xmrig/blob/master/LICENSE) and [XMRig-Proxy](https://github.com/xmrig/xmrig-proxy/blob/master/LICENSE) are licensed under the GNU General Public License v3.0.

[Monerod](https://github.com/monero-project/monero) [licence](https://github.com/monero-project/monero?tab=License-1-ov-file)

[See the licenses of various dependencies.](https://github.com/Cyrix126/gupaxx/blob/master/Cargo.toml)

## Mirror
In case Github repository is down, you can still find the source code at [librejo](https://librejo.monerodevs.org/Ecosystem/gupaxx)

## Donations
If you'd like to thank me for the development of Gupaxx and/or motivate me to improve it you're welcome to send any amount of XMR to the following address:

![QR CODE DONATION ADDRESS](assets/donation_qr.png)
```
4AGJScWSv45E28pmwck9YRP21KuwGx6fuMYV9kTxXFnWEij5FVEUyccBs7ExDy419DJXRPw3u57TH5BaGbsHTdnf6SvY5p5
```

Every donations will be converted to hours of work !

### Donation transparency

A Kuno page exist so you can easly keep track of the amount funded in this project.  
[Gupaxx Kuno](https://kuno.anne.media/fundraiser/dsrr/)  
In case you don't want to rely on the kuno website, the secret view key is:  

```
6c6f841e1eda3fba95f2261baa4614e3ec614af2a97176bbae2c0be5281d1d0f
```
