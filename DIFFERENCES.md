# Differences with upstream [Gupax](https://github.com/hinto-janai/gupax)

This document reference the biggest changes from Gupax. It will not go into every details that you can see in the different releases changelog.


## Added functionalities
### Integration of the XvB Raffle
A new fancy tab to apply an algorithm of distribution of HR to XMRig (see [NOTES_ALGORITHM](NOTES_ALGORITHMS)) with your token from XvB.  
This tab also includes a console output that lets you track if everything is working and see what the decision are of the algorithm, and show you your personal stats from XvB.

A new column in Status Tab to see public stats from the raffle.
### XMRig-Proxy
You can now point all your external miners and get all the juicy stats in Gupaxx. XvB algorithm is able to control XMRig-Proxy when it is enabled.

### Monerod
A new tab for a monero node integration. It allows you to start monerod from gupaxx and benefit from an easy setup with p2pool.

## Removed functionality
Updates by tor. The version of the crate used was outdated, plagued with security concerns and bloated the binary.  
It was only for updates.  
If you want Gupaxx to update by tor, you can torify it when launching.

## Technical Debt
All dependencies are upgraded to last possible version, even when there is a breaking change (code of Gupaxx is modified for that).

## Bugfixes (visuals and performances)
The rendering of Tabs has been modified so that the minimum stated size of the window is capable to show everything. In Upstream the middle panels often overlap on the bottom.

The rendering of the benchmark table and of console outputs were calculating every line at the same time. Now it only renders what you see. It is a significant improvement for your processor, and you can feel the difference if it is not very powerful.

Updates from Gupaxx does not retrieve XMRig and P2Pool from upstream anymore, but use versions in the bundled version. This modification prevent bad surprise (see [#3](https://github.com/Cyrix126/gupaxx/issues/3)).

It also allows advanced users to use your their own version of P2Pool and XMRig. The standalone version of Gupaxx will not replace them.

pings of remote p2pool nodes are much faster.

## UI

### Text size

The fonts size has been rethinked to enable you to use Gupaxx on different size of screen. Before, the size of text was tied to the size of the window. You could not show more content by making the window bigger. Now the size of the text remains the same, except if you change it in the Gupaxx tab.

### Hidden tab and columns

Tabs and column from the Status tab can be hidden, to let you see only what you use in Gupaxx.  

### Daemon mode

Gupaxx can be started in CLI only environment without a GUI. You can do so by starting it with the argument: `--daemon`

## Security
Gupaxx is updated frequently and there is a CI action to check for known vulnerabilities (cargo audit). 
