<h1 align="center">∞ &nbsp; Garlic Alpha</h1>
<p align="center"><a href="README.md">English</a> · Русский · <a href="README.zh-CN.md">简体中文</a></p>

VPN для Android и Windows на собственных Linux-серверах. Вы выбираете, где клиенты входят в сеть и через какой узел выходят в интернет.

**[Скачать для Windows x64](https://github.com/ifmain/Garlic/releases/latest/download/Garlic-Alpha-Windows-x64.exe) · [Скачать для Android](https://github.com/ifmain/Garlic/releases/latest/download/garlic-alpha.apk) · [Сборка из исходников](docs/build.md)**

## ∞ Infinity

Когда приложение открывает соединение, Garlic запрашивает поток на выходном узле. Клиент может подключиться к выходу напрямую или через входной узел. Выход учитывает открытые потоки каждого пользователя и оставляет места для остальных. Уже открытые соединения сохраняют свои места.[^wire]

- Один серверный бинарник запускается в роли входного, промежуточного или выходного узла. [Настройка Linux](deploy/linux/README.md)
- Клиенты Android и Windows используют общее ядро на Rust и согласуют с сервером GEX2, GEX3 или экспериментальный GEX4. После горячей замены модуля новые соединения используют обновление, а открытые продолжают работать с прежним модулем. [Формат модулей](docs/protocol-module.md)

> [!TIP]
> По умолчанию оба клиента получают стабильные релизы. Для тестирования выберите «Релизы и пререлизы» в настройке «Источник обновлений».

## Исследования и авторы

Для профиля трафика Garlic использует размеры, направления и относительное время зашифрованных TLS-записей из указанного ниже набора данных. При подготовке профиля адреса и содержимое пакетов удалены.[^data]

Автор Garlic благодарит **[2dust][2dust]** за код, использованный в интеграции X-UI. Для совместимости с панелью использовался проект **[3x-ui][panel]**. В клиенты также входят модифицированные ipstack и tun2proxy. [Уведомления и лицензии](apps/windows/runtime/assets/licenses)

## Участие в проекте

Буду рад отзывам, обсуждениям и pull request. Если хотите изменить поведение клиента, начните с SDK и примеров Pong и Dev Connection. [Предложить улучшение](https://github.com/ifmain/Garlic/issues) · [Garlic Plugins](https://github.com/ifmain/Garlic-Plugins)

Собственный код Garlic распространяется по [Apache License 2.0](LICENSE). У стороннего кода и данных сохраняются их лицензии.

[^wire]: [Жизненный цикл потоков и лимиты пользователей](crates/coordinator/PROTOCOL.md)
[^data]: Antonio Muñoz, Sergio López Flores, Juan Carlos Jiménez Ariza. *Benign Network Traffic PCAP Dataset for Reproducibility* (2026), v1. [Набор данных](https://doi.org/10.5281/zenodo.19206234) · [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) · [Атрибуция и описание обработки](apps/android/app/assets/traffic-profile-notice.txt)

[2dust]: https://github.com/2dust
[panel]: https://github.com/MHSanaei/3x-ui

