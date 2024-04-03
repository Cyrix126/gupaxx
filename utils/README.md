## Utilities

| File/Folder | Purpose |
|-------------|---------|
| prepare.sh  | Changes version across repo, commits README.md + CHANGELOG.md
| create_tmp_env.sh | create temporary directory to be used for packaging.
| move_binary_inside.sh | once github CI finished, download zip inside temporary file and execute this script.
| package.sh  | Package the contents of `skel`, sign, etc. Checks if all files exist and have the proper naming schemes
| skel        | A skeleton directory with the proper naming scheme + folder structure for packaging Gupax for all OS's
