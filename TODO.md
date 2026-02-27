# TODO: LED BOX 機能拡張ロードマップ

ESP32-C3 + 8x8 LEDマトリクス + ボタン2つのデバイスを、BLEモード切替対応の多機能デスクトップデバイスに拡張する。

## 推奨実装順序

```
Phase 0（基盤）→ Phase 1（ペット）→ Phase 3（ツール）→ Phase 2（タイマー）
    → Phase 4（Claude Code通知）→ Phase 5（Wi-Fi通知）→ Phase 6（スマートホーム）→ Phase 7（モニター）
```

- Phase 0 は全機能の前提となるボタン入力・モード管理・アニメーション基盤
- Phase 1（ペット）はアニメーションエンジンの実戦投入で、基盤の品質を検証できる
- Phase 3（ツール）は最もシンプルで、モード切替の動作確認に最適
- Phase 2（タイマー）はタイマー管理が加わるが、ネットワーク不要
- Phase 4（Claude Code通知）はBLE経由のPC連携で、Wi-Fi不要
- Phase 5以降はWi-Fi/MQTT等の外部連携が必要になり複雑度が上がる

---

## Phase 0: 基盤整備（全機能の前提）

- [x] **0-1: ボタン入力の実装**
  - GPIO割り当て（2ボタン分）
  - デバウンス処理
  - 短押し/長押しの判定
  - 現在ボタン処理は一切なし

- [x] **0-2: モードシステムの設計・実装**
  - `enum Mode { Pet, Pomodoro, SmartHome, Notification, Monitor, Tools }` のようなモード管理
  - 現在のモードに応じてメインループの振る舞いを切り替える

- [x] **0-3: BLEプロトコルの再設計**
  - モード切替用のCharacteristicを追加（または既存Characteristicのプロトコルを拡張）
  - 例: 先頭1バイト=コマンド種別（モード切替/データ送信等）

- [x] **0-4: main.rsのリファクタリング**
  - 現在main.rsにインラインで書かれているBLE初期化を `bluetooth.rs` に統合
  - モジュール分離を整理

- [x] **0-5: アニメーションエンジンの実装**
  - フレームベースのアニメーション再生機能
  - `Vec<[u8; 8]>` のフレーム列を指定間隔で切り替え表示
  - 複数モードで共通利用

- [x] **0-6: ドット絵アセットの管理**
  - 顔、天気アイコン、数字、矢印などのパターンを `const` で定義するモジュール（`src/assets.rs` 等）

## Phase 1: デスクトップ・コンパニオン（デジタルペット）

- [x] **1-1: アイドルアニメーション**
  - 瞬き・キョロキョロなどのランダムアニメーション
  - タイマーで一定間隔ごとに表情を切り替え

- [x] **1-2: ボタンインタラクション**
  - 赤ボタン → 「ご飯（喜ぶ顔）」
  - 白ボタン → 「つつく（怒る顔）」
  - 押下後に数秒間リアクション表示 → アイドルに戻る

- [x] **1-3: 内部ステート管理**
  - 満腹度・機嫌などのパラメータ
  - 時間経過で減少し、表情に影響（お腹が減ると悲しい顔）

## Phase 2: ポモドーロタイマー

- [x] **2-1: タイマーロジック**
  - 25分/5分のカウントダウン
  - 作業 → 休憩 → 作業のサイクル管理

- [x] **2-2: プログレス表示**
  - 8x8=64マスを25分(1500秒)で割り、約23秒ごとに1マスずつ消灯する視覚的カウントダウン

- [x] **2-3: ボタン操作**
  - 赤ボタン → 開始/一時停止
  - 白ボタン → リセット

- [x] **2-4: 完了通知**
  - タイマー終了時にLED全体点滅アニメーション

## Phase 3: アナログツール・ゲーム

- [x] **3-1: 電子サイコロ**
  - 赤ボタン押下でランダムに1〜6の目を表示
  - スロット風アニメーション（約1.5秒の減速演出）
  - 乱数はESP32のハードウェアRNG使用

- [x] **3-2: BLE経由カスタムパターン表示**
  - 従来機能の継承
  - BLEから任意の8バイトパターンを送信して自由表示

- [x] **3-3: BLE経由サブモード切替**
  - 新規Characteristic（UUID: 681285a6-247f-48c6-80ad-68c3dce18587）
  - サイコロ(0) / カスタム表示(1) をBLEから切替可能

## Phase 4: Claude Code 通知モード（BLE連携）

Claude CodeのHooks機能を利用し、応答完了や入力待ちをLED BOXに通知する。

- [x] **4-1: NotificationHandler実装**
  - `data[0] = 0x01`（入力待ち）→ ベルアイコン点滅ループ
  - `data[0] = 0x02`（応答完了）→ チェックマーク表示→消灯ワンショット
  - 赤ボタン短押しで通知クリア
  - 手動でNotificationモードに切替時のみ動作

- [x] **4-2: PC側BLEクライアント**
  - Python + bleak によるBLE書き込みスクリプト（`tools/ble-notify.py`）
  - `python3 ble-notify.py waiting` / `python3 ble-notify.py complete`

- [x] **4-3: Claude Code Hooks設定例**
  - `Notification`イベント → `ble-notify.py waiting`
  - `Stop`イベント → `ble-notify.py complete`
  - 設定例: `tools/claude-hooks-example.json`

## Phase 5: 天気表示（Wi-Fi連携）

ESP32-C3の単一ラジオでWi-FiとBLEをTDM（時分割多重）で同時利用し、天気情報をLEDに表示する。

### 技術的前提
- Wi-Fi STA + NimBLE BLEの同時利用は`CONFIG_ESP_COEX_SW_COEXIST_ENABLE=y`で対応可能
- メモリ: Wi-Fi(~80-100KB) + NimBLE(~60-80KB) + システム(~50-60KB) ≒ 220-290KB使用、残り110-180KBでアプリ動作可能
- **TLSは使わない**: HTTPS時のTLSハンドシェイクで追加40-50KBのヒープが必要なため、HTTP対応APIを使用
- 天気API: **Open-Meteo**（API key不要、HTTP対応、レスポンス約520バイト、WMO天気コード）

### クリティカルパス
- `peripherals.modem`の所有権: `EspWifi::new()`と`esp32-nimble`が競合する可能性 → 要プロトタイピング
- `BlockingWifi`の接続ブロック（数秒）→ ローディングアニメーションで対応

### タスク

- [ ] **5-1a: Wi-Fi + BLE共存基盤**
  - sdkconfig追加: `CONFIG_ESP_COEX_SW_COEXIST_ENABLE=y`、Wi-Fiバッファ削減、`CONFIG_COMPILER_OPTIMIZATION_SIZE=y`
  - `BlockingWifi` + `EspWifi::new()`でSTA接続
  - `esp32-nimble`との`peripherals.modem`共存確認（最優先で検証）
  - ランタイムヒープモニタリング（`heap_caps_get_free_size`）で余裕を確認

- [ ] **5-1b: Wi-Fiクレデンシャル管理**
  - NVS namespace `"wifi_cfg"` にSSID/パスワードを保存・読出し
  - 初期は環境変数 or ソースコードハードコードで開発、後にBLE経由設定を検討

- [ ] **5-2: HTTPクライアントで天気取得**
  - `esp_idf_svc::http::client::EspHttpConnection`でHTTP GET
  - エンドポイント: `http://api.open-meteo.com/v1/forecast?latitude=35.68&longitude=139.77&current=temperature_2m,weather_code`
  - 1KBバッファでレスポンス受信
  - JSONパースは手動パターンマッチ（`serde_json`はバイナリサイズ増加のため避ける）
  - WMO天気コード → アイコン種別へのマッピング

- [ ] **5-3a: 天気アイコンのドット絵アセット作成**
  - `assets.rs`に追加: `ICON_SUN`(太陽), `ICON_CLOUD`(雲), `ICON_RAIN`(雨), `ICON_SNOW`(雪), `ICON_THUNDER`(雷), `ICON_FOG`(霧)
  - 雨・雪はアニメーション用の複数フレーム
  - `ICON_WIFI_ERROR`(接続エラー), `ICON_LOADING`(回転アニメーション)

- [ ] **5-3b: WeatherHandler実装**
  - `src/handlers/weather.rs` 新規作成
  - `Mode` enumに`Weather`を追加
  - 状態遷移: Connecting → Fetching → Displaying / Error
  - 10〜15分間隔で自動更新
  - ボタン操作: 赤短押し=即時更新、赤長押し=気温表示切替
  - WMO天気コード → アイコンマッピング（0=晴, 1-3=曇, 51-55=霧雨, 61-65=雨, 71-75=雪, 80-82=にわか雨, 95+=雷雨）

- [ ] **5-3c: 気温表示（オプション）**
  - 赤ボタン長押しで気温を数字表示（2桁 + 度マーク）
  - 数秒後に天気アイコンに自動復帰

### 実装順序
```
5-1a（Wi-Fi+BLE共存確認）→ 5-1b（NVS設定）→ 5-2（HTTP天気取得）
  → 5-3a（アセット）→ 5-3b（ハンドラー）→ 5-3c（気温表示）
```

### 旧5-4（汎用通知受信）について
BLE経由の通知はPhase 4のNotificationHandlerで既にカバーされているため、Phase 5からは除外。
Wi-Fi経由の汎用通知が必要になった場合はPhase 6以降で検討する。

## Phase 6: スマートホーム・コントローラー

ESP32-C3のWi-Fi経由でHome Assistant等のスマートホームシステムと連携し、物理ボタンによるアクション発火とLED状態表示を実現する。

### 技術的前提

#### プロトコル選定: MQTT（plain TCP）

調査の結果、以下の理由でMQTTを採用する。

| プロトコル | RAM | Rust対応 | 双方向 | HA連携 | 判定 |
|---|---|---|---|---|---|
| **MQTT（plain TCP）** | ~20-25KB | `esp-idf-svc` v0.51に内蔵 | Yes | Auto-Discovery対応 | **採用** |
| HTTP Webhook | ~5KB | 同上 | No（片方向） | Webhook trigger対応 | 補助的に検討 |
| Matter over Wi-Fi | ~195KB | Rust未サポート（`esp-idf-matter` v0.0.0） | Yes | 対応 | 不採用 |
| Thread/Zigbee | N/A | N/A | N/A | N/A | ESP32-C3に802.15.4ラジオなし。物理的に不可 |
| HomeKit | ~120KB | Rust未サポート（C SDKのみ） | Yes | HA経由 | 不採用 |

- **MQTT クライアント**: `esp_idf_svc::mqtt::client::EspMqttClient`（追加依存不要）
- **ブローカー**: Home Assistant公式のMosquitto add-on（port 1883、plain TCP）
- **TLSは使わない**: TLSハンドシェイクで追加40-50KBのヒープが必要。LAN内通信では不要
- **ターゲットプラットフォーム**: Home Assistant（DIYスマートホームユーザーの事実上の標準。2025年時点で200万以上のアクティブインストール）

#### メモリバジェット

```
ESP32-C3 SRAM合計:              400KB
Wi-Fi STA:                      -80KB（削減バッファ設定）
NimBLE (BLE):                   -65KB
FreeRTOS + システム:             -55KB
MQTT client (plain TCP):        -25KB
──────────────────────────────────
アプリケーション利用可能:         ~175KB ✓
```

#### MQTT API パターン

`EspMqttClient::new_cb`（コールバック方式）を採用。既存のシングルスレッドメインループと統合しやすい。

```rust
let mut client = EspMqttClient::new_cb(
    "mqtt://192.168.1.x:1883",
    &MqttClientConfiguration {
        client_id: Some("led_box"),
        buffer_size: 256,        // メッセージが小さいため削減
        out_buffer_size: 256,
        task_stack: 4096,        // デフォルト6144から削減
        lwt: Some(LwtConfiguration {
            topic: "led_box/status",
            payload: b"offline",
            qos: QoS::AtLeastOnce,
            retain: true,
        }),
        ..Default::default()
    },
    move |event| { /* コールバック処理 */ },
)?;
```

**ESP32-C3固有の注意点**: `Connected`コールバック発火前に`subscribe()`を呼ぶとパニックする（esp-idf-svc#419）。`Connected`イベント後にフラグを立て、メインループ側で`subscribe`を実行すること。

#### Home Assistant MQTT Auto-Discovery

デバイス登録時に以下のDiscoveryメッセージを`retain=true`でpublishする。HAが自動的にLED Boxをデバイスとして認識し、UIに表示する。

**デバイストリガー（ボタン → HAオートメーション）:**
```
Topic: homeassistant/device_automation/led_box/btn_red_short/config
Payload:
{
  "automation_type": "trigger",
  "type": "button_short_press",
  "subtype": "button_red",
  "topic": "led_box/button/red/action",
  "payload": "short_press",
  "device": {
    "identifiers": ["led_box_001"],
    "name": "LED Box",
    "model": "LED Box v1",
    "manufacturer": "Custom"
  }
}
```

同様に `btn_red_long`, `btn_wht_short`, `btn_wht_long` の計4つのトリガーを登録。

**センサー（現在モード表示）:**
```
Topic: homeassistant/sensor/led_box_mode/config
Payload: { "name": "LED Box Mode", "unique_id": "led_box_mode",
           "state_topic": "led_box/mode", "icon": "mdi:cube-outline",
           "device": { "identifiers": ["led_box_001"] } }
```

#### MQTTトピック一覧

```
PUBLISH（ESP32 → ブローカー、retained）:
  homeassistant/device_automation/led_box/btn_red_short/config   -- Discovery
  homeassistant/device_automation/led_box/btn_red_long/config    -- Discovery
  homeassistant/device_automation/led_box/btn_wht_short/config   -- Discovery
  homeassistant/device_automation/led_box/btn_wht_long/config    -- Discovery
  homeassistant/sensor/led_box_mode/config                       -- Discovery
  led_box/status                                                 -- "online"/"offline" (LWT)

PUBLISH（ESP32 → ブローカー、not retained）:
  led_box/button/red/action                                      -- "short_press"/"long_press"
  led_box/button/white/action                                    -- "short_press"/"long_press"
  led_box/mode                                                   -- "Pet"/"Pomodoro"等

SUBSCRIBE（ESP32 ← ブローカー）:
  led_box/command/#                                              -- HAからの表示コマンド
```

#### 設計方針

- **SmartHomeモードはopt-in**: MQTTブローカー未設定時はモード一覧から除外 or エラーアイコン表示
- **Lazy接続**: SmartHomeモードに入った時だけMQTT接続。離脱時に切断（~25KB RAM節約）
- **NVSにMQTT設定を永続化**: BLE経由でブローカーIP・認証情報を設定可能に

### タスク

- [ ] **6-1: MQTTクライアント基盤**
  - `esp_idf_svc::mqtt::client::EspMqttClient::new_cb`でMQTT接続
  - SmartHomeモード進入時にconnect、離脱時にdisconnect（Lazy接続）
  - `Connected`イベント後にsubscribe（ESP32-C3パニック回避）
  - Last Will: `led_box/status` = `"offline"`（retain=true）
  - 接続成功時: `led_box/status` = `"online"`（retain=true）
  - バッファサイズ256B、タスクスタック4096Bに削減（メモリ節約）
  - `Arc<Mutex<>>`でコールバックからメインループへデータ共有

- [ ] **6-2: MQTT設定のNVS管理 + BLEプロビジョニング**
  - NVS namespace `"smarthome"` に保存:
    - `mqtt_host`: ブローカーIPアドレス（例: "192.168.1.100"）
    - `mqtt_port`: ポート番号（デフォルト1883）
    - `mqtt_user`: 認証ユーザー名（オプション）
    - `mqtt_pass`: 認証パスワード（オプション）
  - BLE Characteristicから設定を書き込み可能にする
  - 設定未完了時はSmartHomeモードでエラーアイコン表示

- [ ] **6-3: HA Auto-Discoveryとボタンイベント送信**
  - MQTT接続後にDiscovery configをpublish（4つのdevice_automation + 1つのsensor、全てretain=true）
  - ボタン押下時に対応するアクショントピックへpublish:
    - 赤短押し → `led_box/button/red/action` = `"short_press"`
    - 赤長押し → `led_box/button/red/action` = `"long_press"`
    - 白短押し → `led_box/button/white/action` = `"short_press"`
    - 白長押し → `led_box/button/white/action` = `"long_press"`
  - モード切替時に `led_box/mode` へ現在モード名をpublish

- [ ] **6-4: SmartHomeHandler実装 + 状態表示アイコン**
  - `src/handlers/smarthome.rs` 新規作成
  - `Mode` enumに`SmartHome`を追加
  - 状態遷移: Disconnected → Connecting → Connected → Error
  - 接続中はローディングアニメーション（Phase 5のものを共用）
  - 接続完了後はホームアイコン表示
  - HAからの `led_box/command/icon` subscribeで表示アイコンを切替可能
  - HAからのフィードバック受信時に一時的にアイコン変更（チェックマーク等、2秒後に復帰）

- [ ] **6-5: アセット追加**
  - `assets.rs`に追加: `ICON_HOME`（家）, `ICON_MQTT_OK`（接続成功）, `ICON_MQTT_ERR`（接続エラー）
  - 既存の`ICON_CHECK`, `ICON_LOADING`は共用可能

### 実装順序
```
Phase 5（Wi-Fi基盤）完了が前提
  → 6-1（MQTTクライアント基盤）→ 6-2（NVS設定 + BLEプロビジョニング）
    → 6-3（Auto-Discovery + ボタンイベント）→ 6-5（アセット）→ 6-4（ハンドラー実装）
```

### ユースケース例

2ボタン × 短押し/長押し = 4アクション。HAのオートメーションで自由に割り当て可能。

| ボタン | 操作 | HAオートメーション例 |
|---|---|---|
| 赤 短押し | リビング照明トグル | `light.toggle` → `light.living_room` |
| 赤 長押し | 映画モードシーン起動 | `scene.turn_on` → `scene.movie_mode` |
| 白 短押し | デスクライトトグル | `light.toggle` → `light.desk_lamp` |
| 白 長押し | おやすみシーン起動 | `scene.turn_on` → `scene.goodnight` |

### HAオートメーション設定例（ユーザー側）

```yaml
automation:
  - alias: "LED Box - リビング照明トグル"
    trigger:
      - platform: device
        domain: mqtt
        device_id: <auto-assigned>
        type: button_short_press
        subtype: button_red
    action:
      - service: light.toggle
        target:
          entity_id: light.living_room
```

## Phase 7: PCパフォーマンスモニター

- [ ] **7-1: BLEデータ受信プロトコル拡張**
  - PC側からCPU使用率やミュート状態等のデータを受信するフォーマット定義

- [ ] **7-2: バーグラフ表示**
  - CPU使用率を8段階の縦バーグラフで表示

- [ ] **7-3: ミュートインジケーター**
  - マイクON/OFFのアイコン表示
  - ボタン押下でPC側にミュートトグル通知を送信

- [ ] **7-4: PC側コンパニオンアプリ**
  - BLE経由でシステム情報を送信するデスクトップアプリ（Python/Rust等）
  - 別リポジトリ推奨
