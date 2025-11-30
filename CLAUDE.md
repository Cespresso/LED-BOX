# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

ESP32-C3マイコン向けのRust製LEDマトリクスディスプレイ制御プロジェクト。BLE（Bluetooth Low Energy）経由でLEDパターンを受信し、8x8 LEDマトリクス（MAX7219ドライバ使用）に表示する。

## ビルド・実行コマンド

```bash
# メインプログラムのビルドと書き込み
cargo run

# exampleの実行（.rsは不要）
CRATE_CC_NO_DEFAULTS=1 cargo run --example nvs
```

## 開発環境

- ターゲット: `riscv32imc-esp-espidf`（ESP32-C3）
- ESP-IDF: v5.2.2
- Rustツールチェイン: nightly
- フラッシュツール: `espflash`（書き込みとモニタリング）

## アーキテクチャ

### ハードウェア構成
- MCU: ESP32-C3
- LEDマトリクス: MAX7219ドライバ（SPI接続）
  - GPIO8: SCLK
  - GPIO9: CS
  - GPIO10: MOSI

### モジュール構成
- `src/main.rs` - エントリポイント。BLEサーバー初期化、メインループでLEDマトリクス更新
- `src/utils/led.rs` - SPI初期化とMAX7219マトリクスディスプレイの設定
- `src/utils/bluetooth.rs` - BLEサーバー設定とCharacteristic管理

### データフロー
1. BLE Characteristicにクライアントから8バイトのデータを書き込み
2. データはNVS（不揮発性ストレージ）に永続化
3. メインループで2秒ごとにCharacteristicの値を読み取り
4. 8バイトデータをMAX7219経由でLEDマトリクスに送信

### BLE設定
- サービスUUID: `455aa9f0-2999-43de-81b4-54e0de255927`
- Characteristic UUID: `681285a6-247f-48c6-80ad-68c3dce18585`
- デバイス名: "LED BOX"
- パスキー: 123456
