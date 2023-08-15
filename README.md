# Nostr flow meter on Your Relay Server

自身のリレーサーバー上でローカル稼働させ、流速ちゃんからの計測に頼らず自らのサーバーの流速情報を提供できます。

## 要件

流速情報は kind: 30078 (NIP-78) を使用して自身のリレー上に保持します。

### 連続集計

テーブル名: `frowmeter_[relay_key]`

```json
{
  "kind": 30078,
  "content": "test",
  "tags": [
    [ "d", "frowmeter_shino3" ],
    [ "title", "frowmeter_shino3" ],
    [ "t", "frowmeter_shino3" ],
    [ "202308151013", "0" ],
    [ "202308151014", "0" ],
    [ "202308151015", "0" ],
    [ "202308151016", "0" ]
  ],
  "created_at": "unix-time"
}
```

### 日別集計

テーブル名: `frowmeter_[relay_key]_[date]`

```json
{
  "kind": 30078,
  "content": "test",
  "tags": [
    [ "d", "frowmeter_shino3_20230815" ],
    [ "title", "frowmeter_shino3_20230815" ],
    [ "t", "frowmeter_shino3_20230815" ],
    [ "202308151013", "0" ],
    [ "202308151014", "0" ],
    [ "202308151015", "0" ],
    [ "202308151016", "0" ]
  ],
  "created_at": "unix-time"
}
```

## 導入方法

※サーバー上で Node.JS v18 が動くように設定してください。

### 初回セットアップ

```bash
git clone https://github.com/ShinoharaTa/nostr-flowmeter-batch-on-server.git
cd ./nostr-flowmeter-batch-on-server

npm install
cp .env.sample .env
nano .env

# build
npm run build
```

### アップデート方法

```bash
cd ./nostr-flowmeter-batch-on-server
git ぷっｌ
npm install

# build
npm run build
```

デーモン化などの方法はお好きな方法で。
例: supervisor等
