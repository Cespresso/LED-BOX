# LED BOX

ESP32-C3 (Seeed Studio XIAO ESP32C3) + 8x8 LEDマトリクス (MAX7219) + ボタン2つの多機能デスクトップデバイス。
BLE経由でモード切替・表示データ送信が可能。

## ビルド・書き込み

```bash
# メインプログラム
cargo run

# exampleの実行（.rsはつけない）
CRATE_CC_NO_DEFAULTS=1 cargo run --example ファイル名
```

## 書き込みできない場合（ブートローダーモードへの入り方）

GPIO 18/19 (USB) を誤って再設定した場合など、通常の書き込みができなくなることがあります。
その場合、手動でブートローダーモードに入る必要があります。

### ボタン配置

```
        USB-C端子
     ┌──[=====]──┐
     │            │
     │   ESP32    │
     │    C3      │
     │            │
     │ [B]   [R]  │
     └────────────┘
      BOOT  RESET
```

- USB-C端子の**反対側の端**に2つの小さなボタンがあります
- 左側: **BOOT** ボタン (GPIO9)
- 右側: **RESET** ボタン

### 手順

1. **BOOT（左）を押し続ける**
2. その状態で **RESET（右）を1回押して離す**
3. **BOOT を離す**
4. `cargo run` で書き込む

または:

1. USBケーブルを**抜く**
2. **BOOT（左）を押し続けたまま** USBを接続
3. **BOOT を離す**
4. `cargo run` で書き込む

### 注意: GPIO 18/19 を使わないこと

ESP32-C3 の GPIO 18/19 は USB D-/D+ です。
これらを `gpio_config` 等で再設定するとUSB接続が切れ、以降通常の書き込みができなくなります。

## BLE仕様

- デバイス名: `LED BOX`
- パスキー: `123456`

### Service

| 項目 | 値 |
|------|-----|
| Service UUID | `455aa9f0-2999-43de-81b4-54e0de255927` |

### Characteristics

#### Mode Characteristic

モードの読み取り・切替・変更通知を行う。

| 項目 | 値 |
|------|-----|
| UUID | `681285a6-247f-48c6-80ad-68c3dce18586` |
| Properties | READ, WRITE, NOTIFY |
| Value | 1 byte |

**モード値:**

| 値 | モード |
|----|--------|
| `0x00` | Pet（デジタルペット） |
| `0x01` | Pomodoro（ポモドーロタイマー） |
| `0x02` | Tools（サイコロ・YES/NO等） |
| `0x03` | Notification（通知インジケーター） |
| `0x04` | SmartHome（スマートホーム） |
| `0x05` | Monitor（PCモニター） |

**操作:**

- **Write**: 1バイト（`0x00`〜`0x05`）を書き込むとモードが切り替わる。範囲外の値は無視される。
- **Read**: 現在のモード値を返す。
- **Notify**: モードが変更されたとき（BLE経由・ボタン操作問わず）、新しいモード値が通知される。Subscribeしておくことでクライアント側がモード変更をリアルタイムに検知できる。

#### Display Data Characteristic

8x8 LEDマトリクスの表示データを送受信する。

| 項目 | 値 |
|------|-----|
| UUID | `681285a6-247f-48c6-80ad-68c3dce18585` |
| Properties | READ, WRITE |
| Value | 8 bytes |

**データフォーマット:**

8バイト = 8行分のLEDデータ。各バイトの各ビットが1ピクセルに対応する（MSB=左端, LSB=右端）。

```
Byte 0: Row 1  [bit7=左端 ... bit0=右端]
Byte 1: Row 2
...
Byte 7: Row 8
```

**操作:**

- **Write**: 8バイトを書き込むと表示データが更新される（Toolsモード等で使用）。
- **Read**: 現在の表示データを返す。

### クライアント実装例

```python
# Python (bleak) でのモード切替例
import asyncio
from bleak import BleakClient

MODE_UUID = "681285a6-247f-48c6-80ad-68c3dce18586"
DISPLAY_UUID = "681285a6-247f-48c6-80ad-68c3dce18585"

async def main():
    async with BleakClient("LED BOX") as client:
        # 現在のモードを読み取り
        mode = await client.read_gatt_char(MODE_UUID)
        print(f"Current mode: {mode[0]}")

        # Petモード(0)に切替
        await client.write_gatt_char(MODE_UUID, bytes([0x00]))

        # モード変更の通知を受け取る
        def on_mode_change(sender, data):
            print(f"Mode changed to: {data[0]}")
        await client.start_notify(MODE_UUID, on_mode_change)

        # 表示データを送信（スマイル顔）
        smile = bytes([0x3C, 0x42, 0xA5, 0x81, 0xA5, 0x99, 0x42, 0x3C])
        await client.write_gatt_char(DISPLAY_UUID, smile)
```

## Claude Code 通知モード セットアップ

Claude Codeの応答完了や入力待ちをLED BOXで通知する。

### 前提

- LED BOXがNotificationモードになっていること（Androidアプリからモード切替: `0x03`）
- Python 3 + bleak がインストール済み

```bash
pip install bleak
```

### 1. 動作確認

```bash
# 入力待ち通知（ベルアイコン点滅）
python3 tools/ble-notify.py waiting

# 応答完了通知（チェックマーク表示）
python3 tools/ble-notify.py complete
```

### 2. Claude Code Hooks 設定

`~/.claude/settings.json` に以下を追加:

```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "python3 /absolute/path/to/led-box/tools/ble-notify.py waiting"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "python3 /absolute/path/to/led-box/tools/ble-notify.py complete"
          }
        ]
      }
    ]
  }
}
```

`/absolute/path/to/led-box/` を実際のパスに置き換えること。

### 通知種別

| イベント | LED表示 | クリア方法 |
|---------|---------|-----------|
| `Notification` (入力待ち/権限確認) | ベルアイコン点滅 | 赤ボタン短押し |
| `Stop` (応答完了) | チェックマーク → 自動消灯 | 自動 or 赤ボタン |
