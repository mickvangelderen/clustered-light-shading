Requires SteamVR (installed through steam) and standard OpenGL development
headers and libraries to be installed.

Requires rust (nigtly probably)

Probably only works on unixies.

Run with steam-runtime, for example:

```
~/.steam/steam/ubuntu12_32/steam-runtime/run.sh lldb target/debug/vr-lab
```

Disable vsync in the driver if you're using vr. This prevents the buffer swap
from blocking and thus render at your display's frequency which is probably 60Hz
while we need to render to the HDM at 90Hz.

## Dependencies

### Sibling repositories.

`git clone git@github.com:mickvangelderen/gl-typed-rust`
`git clone git@github.com:mickvangelderen/openvr-sys-rust --recursive`
`git clone git@github.com:mickvangelderen/openvr-rust`

### Steam needs QT5.

Also OpenGL development headers are always nice to have.

`sudo apt install qtbase5-dev mesa-common-dev`

## Troubleshooting

1. If SteamVR is running and *then* you plug in the HMD, it will not work, at
   least on Linux by Jan 2019. Kill the SteamVR processes and start the
   application.

2. If SteamVR is not running and you start the application, it might give up
   before SteamVR has completely launched and exit. Starting the application
   again might make things work because SteamVR had time to launch.
   
3. If you're using a single base-station, make sure it is configured as type A.

4. If somethings up with the config path:
   https://github.com/ValveSoftware/SteamVR-for-Linux/issues/89

## Interesting stuff

Reducing render-to-photons latency: http://media.steampowered.com/apps/valve/2015/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf


