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


### Pairing clients with a private network CA

Use a DNS name you control for the initial connection. Serve the public network CA certificate in DER format at `https://<entry-host>/.well-known/garlic/ca.der` on port 443. That HTTPS virtual host needs a certificate trusted by the client before pairing, with a matching DNS name and its intermediate certificates. The Garlic API and exchange can continue using the private network CA on their existing port. A gateway with an already trusted API certificate works without a private CA download.

For example, convert the network CA's public certificate and serve that one file from the existing HTTPS virtual host:

```sh
install -d -m 0755 /var/lib/garlic-enrollment
openssl x509 -in network-ca.pem -outform DER -out /var/lib/garlic-enrollment/ca.der
chmod 0644 /var/lib/garlic-enrollment/ca.der
```

```nginx
location = /.well-known/garlic/ca.der {
    default_type application/pkix-cert;
    alias /var/lib/garlic-enrollment/ca.der;
}
```

First-time users should enter the enrollment host as their initial server. A fresh client that knows only a different gateway needs enrollment available on that gateway's host too. Existing profiles use the DNS entry node in the saved network directory or configured control routes. The enrollment GET carries no account or device token. Redirects are rejected. Once paired, clients keep the CA in the local profile and verify the gateway hostname and certificate on both the API and exchange paths. The CA does not enter the OS trust store. On an unknown issuer the client can fetch the public file again; keep the enrollment HTTPS certificate independently trusted during CA rotation.
