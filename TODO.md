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

- [ ] **6-1: プリセットアクション定義**
  - ボタンに紐づくアクション（会議モード/終了モード等）をNVSで設定可能に

- [ ] **6-2: MQTT or HTTP連携**
  - Home Assistant等への状態送信
  - ボタン押下時にMQTTパブリッシュ or HTTP POST

- [ ] **6-3: 状態表示アイコン**
  - 現在のモード状態を示すアイコン表示（会議中=×マーク、通常=笑顔等）

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
