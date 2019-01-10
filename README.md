Requires https://github.com/mickvangelderen/openvr-sys-rust as a sibling
directory. Sorry didn't bother with setting up a cargo workspace.

Requires SteamVR (installed through steam). 

Requires rust (nigtly probably)

Probably only works on unixies.

Run with steam-runtime, for example:

```
~/.steam/steam/ubuntu12_32/steam-runtime/run.sh lldb ./vr-lab
```

Probably missed something.

I am really frustrated that things don't work right now.

## Troubleshooting

If SteamVR is running and *then* you plug in the HMD, it will not work, at least
on Linux by Jan 2019. Kill the SteamVR processes and start the application.
