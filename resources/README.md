# Resources

This folder contains all of the resources loaded at run-time by the binaries in
this project.

## Split files

I used to commit files to LFS, but then Github started charging for LFS usage and I could not check out my baby project anymore. I purchased an LFS data pack for 5$ to allow myself to check out the project with essential resources again.

Since this project is largely inactive, I don't want to pay for LFS data packs every month and decided to remove LFS and commit the files into the repo instead, so at least it can be checked out. The resources can be downloaded but in the past 4 years they have changed and were added with a different FBX version (7500 from 7300), requiring changes in the FBX parser.

Files larger than 100MB are rejected by Github and so the files have to be [`unsplit.sh`](./unsplit.sh) after checking out and [`split.sh`](./split.sh) before checking in (if there are any changes to the to-be-split files).

## bistro

Download it from here: https://developer.nvidia.com/orca/amazon-lumberyard-bistro

## sun_temple

Download it from here: https://developer.nvidia.com/ue4-sun-temple
