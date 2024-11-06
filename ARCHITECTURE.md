# Gupaxx ARCHITECTURE

This document explains how the source code is organized. Everything differing from [Gupax](https://github.com/hinto-janai/gupax) is described here. Things that are not changed will not be present.

## Structure
| File/Folder  | Purpose |
|--------------|---------|
|main.rs| Launch the app.
|inits.rs| Launch the threads if auto, including XvB.
|miscs.rs| Useful functions.
|app| Directory with everything related to displaying the UI.
|app/keys.rs| Handle keys input.
|app/mod.rs| Define App struct, used by egui.
|app/eframe_impl.rs| First entry to the UI.
|disk/| Code for writing to disk: `state.toml/node.toml/pool.toml`; This holds the structs for the [State] struct.
|helper| The "helper" thread that runs for the entire duration Gupax is alive. All the processing that needs to be done without blocking the main GUI thread runs here, including everything related to handling P2Pool/XMRig/XvB.
|helper/xvb| All related thread XvB code.
|helper/xvb/mod.rs| XvB thread and principal loop, checks and triggers, gluing every other code of this directory.
|helper/xvb/algorithm.rs| Algorithm logic with calculations and actions.
|helper/xvb/nodes.rs| Manage connection of XvB nodes.
|helper/xvb/rounds.rs| Struct for Rounds with printing and detecting of current round.
|helper/xvb/public\|private_stats| Struct to retrieve public and private stats with request.
|component| Gupaxx related features, like updates and nodes.


## Technical differences of column XMRig in Status Tab process sub menu with upstream Gupax

Status of process for XMRig use for some information an image of data when the process started.
The node of xmrig in upstream can not change without a restart of the process. In this fork, the node used by XMRig needs to be updated without restart (using the config HTTP API of XMRig).
So Gupaxx needs to refresh the value of status tab submenu process for XMRig where before the values could not change without a restart of the process.
The field node from ImgXmrig needs to be moved to PubXvbApi. This value must be updated by XMRig at start and by XvB process at runtime.

## Updates

A new option in Gupaxx tab advanced will enable bundled updates.
The binary included of gupaxx will have default value for bundled updates depending if it is coming from the standalone or the bundled release.

Updates from Gupaxx will do the following differently from upstream:
- Check if using bundled or standalone with state. Update only Gupaxx binary if the latter or xmrig and p2pool from bundle version if the former.
- Prevent user to run updates twice without restart.
- Ask the user to restart Gupaxx.
- Do not verify if file P2Pool or XMRig exist. (so that the update can create them).
