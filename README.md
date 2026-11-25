# Garlic Alpha

Garlic — self-hosted VPN с клиентами для Android и Windows. Клиенты используют общее сетевое ядро на Rust. Один серверный бинарник запускается в роли входной, промежуточной или выходной ноды.

Скачать [Windows x64 EXE](https://github.com/ifmain/Garlic/releases/download/alpha-0.9.5/Garlic-Alpha-Windows-x64.exe) или [Android APK](https://github.com/ifmain/Garlic/releases/download/alpha-0.9.5/garlic-alpha.apk). Windows запрашивает права администратора до открытия приложения. Клиенты предлагают загрузить новую версию при запуске. Windows проверяет подпись выпуска и применяет обновление при следующем запуске. На Android установку подтверждает системный установщик.

## Linux-сервер

Из корня репозитория, с установленными Git, Rust (rustup), C-компилятором и системным линкером:

```sh
cargo build --locked --release -p garlic-coordinator --bins
cargo test --locked -p garlic-relay-data -p garlic-coordinator
```

Сервер находится в `target/release/garlic-server`, утилита управления учётными записями — в `target/release/garlic-auth`. После настройки адресов, TLS и хранилища запустите сервер с нужной ролью, например `garlic-server --role exit`. Сборка Debian-пакета для amd64, конфигурация собственной сети и примеры systemd описаны в [deploy/linux](deploy/linux).

## Windows x64

Требуются Git, Rust для Windows x64, Visual Studio Build Tools с компонентом Desktop development with C++, Windows SDK и .NET Framework 4.x с WPF. Запустите из корня в PowerShell:

```powershell
rustup target add x86_64-pc-windows-msvc
.\apps\windows\build.ps1
```

Для GNU LLVM можно указать `-Target x86_64-pc-windows-gnullvm`. Результат сборки — `dist/Garlic-Alpha-Windows-x64.exe`. Исполняемый файл содержит интерфейс, сетевое ядро и Wintun. Для собственного канала обновлений укажите свой репозиторий в клиентах и замените открытый ключ в `apps/windows/release-public.xml` на свой. Подпись EXE для обновления — отдельный бинарный файл `.exe.sig` (RSA PKCS#1 v1.5, SHA-256).

## Android

Требуются Git, Rust, Python 3.10+, JDK 21, Android SDK Platform 35, Build Tools 35.0.0, NDK 27.3.13750724 и Apktool 3.0.3 (JAR). Установите Android-компоненты через SDK Manager. Укажите пути в переменных окружения:

| Переменная | Путь |
| --- | --- |
| `JAVA_HOME` | Корень JDK |
| `ANDROID_HOME` | Корень Android SDK |
| `ANDROID_NDK_HOME` | Каталог версии NDK внутри SDK |
| `APKTOOL_JAR` | Файл `apktool_3.0.3.jar` |

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi
python apps/android/build.py
```

Сборка компилирует ядро Rust и JNI-библиотеку для ARM64 и ARMv7, собирает интерфейс из Smali, Java и ресурсов в `apps/android`, подписывает APK локальным тестовым ключом и проверяет подпись. Результат — `dist/garlic-alpha.apk`. Сохраните `target/android/debug.p12` для последующих обновлений своей сборки. APK с локальной подписью устанавливается поверх приложения с такой же подписью. Закрытый ключ опубликованного релиза хранится отдельно.

## Где менять код

- [Сетевое ядро клиентов](crates/client), [кодеки](crates/relay-data) и [сессии](crates/relay-session).
- [Сервер, учётные записи и шлюзы](crates/coordinator).
- [Android](apps/android), [Windows](apps/windows) и [JNI](crates/android-bind).
- [Формат подписанного модуля и горячая замена](docs/protocol-module.md), [утилита подписи](tools/sign_profile.py).
- [Интеграционные тесты](tests). Для Windows-теста установите Python-пакет `cryptography`, соберите `garlic-client` с `--features alpha` и бинарники `garlic-coordinator` в debug, затем запустите `python tests/windows_module_reload.py`. Для нестандартного каталога бинарников задайте `GARLIC_FIXTURE_BIN`. Для сравнения с историческим клиентом можно задать `GARLIC_STOCK_CLIENT`.
