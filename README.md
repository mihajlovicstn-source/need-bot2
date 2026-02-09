# solana-vntr-sniper

## Встановлення

### 1) Підготуйте Rust
Встановіть Rust через rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### 2) Сконфігуруйте середовище
Скопіюйте приклад `.env` і заповніть значення:
```bash
cp src/env.example .env
```

Обов’язкові поля:
- `RPC_HTTP` — ваш RPC endpoint (наприклад, Helius).
- `PRIVATE_KEY` — приватний ключ гаманця.
- `COPY_TRADING_TARGET_ADDRESS` — адреса(и) лідера.

RPC polling параметри:
- `RPC_POLLING_ENABLED=true` — увімкнути polling.
- `RPC_POLL_INTERVAL_MS` — інтервал (мс).
- `RPC_POLL_STATE_PATH` — файл для `last_seen` (персистентний стан).

Приклад:
```env
RPC_HTTP=https://rpc.helius.xyz/?api-key=YOUR_KEY
PRIVATE_KEY=YOUR_PRIVATE_KEY
COPY_TRADING_TARGET_ADDRESS=TARGET_PUBKEY
RPC_POLLING_ENABLED=true
RPC_POLL_INTERVAL_MS=300
RPC_POLL_STATE_PATH=.rpc_poll_last_seen
```

### 3) Запуск
```bash
cargo run
```

### 4) Логи
Після запуску бот буде логувати:
- стан ініціалізації,
- RPC polling активність,
- знайдені угоди (trade events).
