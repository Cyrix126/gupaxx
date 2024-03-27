# Differences with upstream [Gupax](https://github.com/hinto-janai/gupax)

## Integration of the XvB Raffle

A new fancy tab to apply an algorithm of distribution of HR to xmrig (see [NOTES_ALGORITHM](NOTES_ALGORITHMS)) with your token from XvB.  
This tab also includes a console output to let you track if everything is working and what are the decision of the algorithm, and show you personal stats from XvB.

A new column in Status Tab to see public stats from the raffle.

## Removed functionality

Updates by tor. The version of the crate used was outdated, plagued with security concerns and bloated the binary.
It was only for updates, and it is not useful for this beta release.
This functionality will be re-added for the stable release in a nicer way.

## Bugfixes (visuals and performances)

The rendering of Tabs has been modified so that the minimum stated size of the window is capable to show everything. In Upstream middles panels often overlap on the bottom.

The rendering of the benchmark table and of console outputs were calculating every line at the same time. Now it only renders what you see. It is a significant improvement for your processor, and you can feel the difference if it is not very powerful.

## Security

With the upgrade of dependencies, cargo audit show no warnings instead of 5 vulnerabilities and 4 allowed warnings. 
