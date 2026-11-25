<h1 align="center">∞ &nbsp; Garlic Alpha</h1>
<p align="center">English · <a href="README.ru.md">Русский</a> · <a href="README.zh-CN.md">简体中文</a></p>

A self-hosted VPN for Android and Windows. Run your own Linux nodes and choose where connections enter and leave your network.

**[Windows x64](https://github.com/ifmain/Garlic/releases/latest/download/Garlic-Alpha-Windows-x64.exe) · [Android](https://github.com/ifmain/Garlic/releases/latest/download/garlic-alpha.apk) · [Build from source](docs/build.md)**

## ∞ Infinity

When an app opens a connection, Garlic requests a stream on an exit node. Clients can reach the exit directly or through an entry node. Each exit tracks how many streams each user has open and reserves room for others. Existing connections keep their slots.[^wire]

- One Linux binary can serve as an entry, intermediate or exit node. [Linux deployment](deploy/linux/README.md)
- Android and Windows share a Rust core. Both negotiate GEX2, GEX3 or experimental GEX4 with the server. After a hot reload, new connections use the updated module while open connections keep their original one. [Module format](docs/protocol-module.md)

> [!TIP]
> Both clients follow stable releases by default. Testers can opt into prereleases in the update source settings.

## Research and credits

Garlic’s traffic profile uses the sizes, directions and relative timing of encrypted TLS records from the dataset below. The derived profile omits addresses and packet contents.[^data]

The maintainer credits **[2dust][2dust]** for code reused in Garlic’s X-UI integration. The upstream panel used for compatibility work is **[3x-ui][panel]**. The clients also include modified versions of ipstack and tun2proxy. [Third-party notices](apps/windows/runtime/assets/licenses)

## Contributing

I welcome feedback, design discussions and pull requests. For client extensions, start with the SDK and the Pong and Dev Connection examples. [Issues](https://github.com/ifmain/Garlic/issues) · [Garlic Plugins](https://github.com/ifmain/Garlic-Plugins)

Garlic’s original code is licensed under [Apache License 2.0](LICENSE). Third-party code and data retain their respective licenses.

[^wire]: [Stream lifecycle and per-user limits](crates/coordinator/PROTOCOL.md)
[^data]: Antonio Muñoz, Sergio López Flores, Juan Carlos Jiménez Ariza. *Benign Network Traffic PCAP Dataset for Reproducibility* (2026), v1. [Dataset](https://doi.org/10.5281/zenodo.19206234) · [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) · [Attribution and processing notes](apps/android/app/assets/traffic-profile-notice.txt)

[2dust]: https://github.com/2dust
[panel]: https://github.com/MHSanaei/3x-ui
