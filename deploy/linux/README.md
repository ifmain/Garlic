# Linux-сервер

## Сборка пакета для Debian и Ubuntu (amd64)

В корне репозитория, на Linux x64 с Rust, Python 3.10+ и пакетом `musl-tools`:

```sh
rustup target add x86_64-unknown-linux-musl
cargo build --locked --release --target x86_64-unknown-linux-musl -p garlic-coordinator --bin garlic-server --bin garlic-auth
python3 deploy/linux/package_deb.py target/x86_64-unknown-linux-musl/release
dpkg-deb --info dist/*.deb
sudo apt install ./dist/*.deb
```

Пакет устанавливает `garlic-server`, `garlic-auth`, пользовательские юниты systemd и образцы настроек в `/usr/share/doc/garlic-server/examples`. Службы запускает администратор после настройки.

## Своя сеть

1. Создайте каталоги состояния и конфигурации для служебного пользователя. Возьмите за основу `entry.env.example` и `exit.env.example`. Замените пути `/home/garlic` и адреса `example.invalid` своими. Для systemd файлы настроек располагаются в `~/.config/garlic/{entry,exit}.env`.
2. Укажите свой сертификат TLS и закрытый ключ для каждой ноды. Для частного центра сертификации у серверов есть `GARLIC_CA_CERT`; для обычного Android-клиента используйте адрес с сертификатом от публично доверенного центра. Ограничьте доступ к закрытым ключам и файлам учётных записей служебным пользователем.
3. Для новой сети инициализируйте пустое хранилище командой `garlic-auth users init ПУТЬ_К_ACCOUNTS_JSON`. Административные операции и параметры первичной настройки определены в [garlic-auth](../../crates/coordinator/src/bin/garlic-auth.rs). Примеры полной инициализации тестовой сети находятся в [интеграционном тесте](../../tests/windows_module_reload.py).
4. Запустите выбранную роль с её переменными окружения. Для systemd используйте `systemctl --user start garlic-entry` или `systemctl --user start garlic-exit`. На клиенте укажите адрес своего сервера.

Для подписанных модулей издатель может встроить открытый Ed25519-ключ в свою клиентскую сборку через `GARLIC_PUBLISHER_KEY_B64` при вызове Cargo (32 байта в стандартном Base64). Без него клиент работает со встроенными протоколами; для отдельной сети также поддерживается локальный публичный ключ. Подробнее в [описании формата](../../docs/protocol-module.md).

## Первый администратор

На новой входной ноде, под её служебным пользователем и после создания каталогов, используйте пути из своего `entry.env`. Пример для Bash:

```bash
set -a
. ~/.config/garlic/entry.env
set +a
umask 077
garlic-auth keys init "$GARLIC_INFRA_KEYS_FILE" primary
garlic-auth users init "$GARLIC_AUTH_FILE"
read -rsp 'Пароль первого администратора: ' garlic_password; echo
printf '%s\n' "$garlic_password" | garlic-auth users add "$GARLIC_AUTH_FILE" admin --password-stdin
unset garlic_password
garlic-auth users role "$GARLIC_AUTH_FILE" admin admin
```

Команды `init` предназначены для первичного создания файлов. Храните собственные ключи и состояние учётных записей отдельно от репозитория. Настройте доверие между своими нодами перед подключением выхода.

Когда входная нода запущена с TLS, можно получить ключ для графического клиента с помощью CLI. На Linux он собирается командой `cargo build --locked --release -p garlic-client --features alpha --bin garlic-internal-network`. Установите адрес своей входной ноды в `GARLIC_CONTROL_URLS`, затем выполните:

```bash
export GARLIC_CONTROL_URLS=https://entry.example.invalid:8443  # замените своим адресом
read -rsp 'Пароль администратора: ' garlic_password; echo
printf '%s\n' "$garlic_password" | garlic-internal-network login "$GARLIC_CONTROL_URLS" admin --password-stdin
unset garlic_password
garlic-internal-network admin key admin
```

Полученный токен и адрес своей ноды введите в Garlic Alpha.
