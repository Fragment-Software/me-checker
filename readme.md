# ME Checker

## English

### Socials

Telegram channel: [https://t.me/fragment_software](https://t.me/fragment_software)

Telegram chat: [https://t.me/fragment_software_chat](https://t.me/fragment_software_chat)

### Installation

#### Prerequisites

- **Rust** : Ensure you have Rust installed. You can download and install Rust from [https://www.rust-lang.org/tools/install]().

### Build

Clone the repository and build the project:

```
git clone https://github.com/Fragment-Software/me-checker.git
cd me-checker
cargo build --release
```

### Configuration

Before running the software, configure the necessary files:

1. **secrets.txt** : Add your private keys to `data/secrets.txt`.
2. **proxies.txt** : Add your proxies to `data/proxies.txt`.
3. **config.toml**: Configure concurrency in `data/config.toml`.

### Running

Execute the built binary:

`cargo run --release`

### Output

After running, the output will be saved to `data/eligible.txt` in the following format:

`wallet_address: allocation`

---

## Русский

### Где нас найти

Telegram channel: [https://t.me/fragment_software](https://t.me/fragment_software)

Telegram chat: [https://t.me/fragment_software_chat](https://t.me/fragment_software_chat)

### Установка

#### Предварительные требования

- **Rust** : Убедитесь, что Rust установлен. Вы можете скачать и установить Rust с [https://www.rust-lang.org/tools/install]().

### Сборка

Клонируйте репозиторий и соберите проект:

```
git clone https://github.com/Fragment-Software/me-checker.git
cd me-checker
cargo build --release
```

### Конфигурация

Перед запуском программного обеспечения настройте необходимые файлы:

1. **secrets.txt** : Добавьте ваши приватные ключи в `data/secrets.txt`.
2. **proxies.txt** : Добавьте ваши прокси в `data/proxies.txt`.
3. **config.toml**: Настройка параллелизма в `data/config.toml`.

### Запуск

Запустите собранный бинарный файл:

`cargo run --release `

### Вывод

После запуска результат будет сохранен в `data/eligible.txt` в следующем формате:

`wallet_address: allocation`
