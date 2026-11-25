<h1 align="center">∞ &nbsp; Garlic Alpha</h1>
<p align="center"><a href="README.md">English</a> · <a href="README.ru.md">Русский</a> · 简体中文</p>

一款适用于 Android 和 Windows 的自托管 VPN。你可以在自己的 Linux 服务器上部署节点，自行决定客户端从哪里接入，以及流量从哪里访问互联网。

**[下载 Windows x64 版](https://github.com/ifmain/Garlic/releases/latest/download/Garlic-Alpha-Windows-x64.exe) · [下载 Android 版](https://github.com/ifmain/Garlic/releases/latest/download/garlic-alpha.apk) · [从源码构建](docs/build.md)**

## ∞ Infinity

应用建立连接时，Garlic 会向出口节点申请一个数据流。客户端既可以直连出口，也可以通过入口节点转发。出口按用户统计已打开的数据流，为其他用户预留名额。已建立的连接会保留原有名额。[^wire]

- 同一个 Linux 服务程序可以作为入口、中继或出口节点运行。 [Linux 部署指南](deploy/linux/README.md)
- Android 和 Windows 客户端共用 Rust 核心，与服务器协商使用 GEX2、GEX3 或实验性的 GEX4。热更新后，新连接使用新模块，现有连接继续使用原模块。 [模块格式](docs/protocol-module.md)

> [!TIP]
> 两个平台默认只接收稳定版。测试者可以在更新来源设置中选择同时接收稳定版和预发布版。

## 研究与致谢

Garlic 的流量配置使用了下列数据集中加密 TLS 记录的长度、方向和相对时间。生成的配置不含网络地址或数据包内容。[^data]

Garlic 作者感谢 **[2dust][2dust]** 提供了用于 X-UI 集成的部分代码。面板兼容性工作参考了 **[3x-ui][panel]**。客户端还使用了经过修改的 ipstack 和 tun2proxy。 [第三方声明与许可证](apps/windows/runtime/assets/licenses)

## 参与贡献

欢迎反馈、讨论设计和提交拉取请求。要开发客户端扩展，可以先看看 SDK 以及 Pong 和 Dev Connection 示例。[提出建议](https://github.com/ifmain/Garlic/issues) · [Garlic Plugins](https://github.com/ifmain/Garlic-Plugins)

Garlic 的原创代码采用 [Apache License 2.0](LICENSE)。第三方代码和数据沿用各自的许可证。

[^wire]: [数据流的生命周期和用户配额](crates/coordinator/PROTOCOL.md)
[^data]: Antonio Muñoz, Sergio López Flores, Juan Carlos Jiménez Ariza. *Benign Network Traffic PCAP Dataset for Reproducibility* (2026), v1. [数据集](https://doi.org/10.5281/zenodo.19206234) · [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) · [署名与数据处理说明](apps/android/app/assets/traffic-profile-notice.txt)

[2dust]: https://github.com/2dust
[panel]: https://github.com/MHSanaei/3x-ui

