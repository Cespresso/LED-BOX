# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## プロジェクト概要

ESP32-C3マイコン向けのRust製多機能デスクトップデバイスプロジェクト。8x8 LEDマトリクス（MAX7219）と2つの物理ボタンを搭載し、BLE経由でモード切替可能な複数機能（デジタルペット、ポモドーロタイマー、通知インジケーター等）を提供する。

詳細なロードマップは `TODO.md` を参照。

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
- ボタン: 2つ（赤・白）※GPIO未割り当て

### モジュール構成
- `src/main.rs` - エントリポイント。BLEサーバー初期化、モード管理、メインループ
- `src/utils/led.rs` - SPI初期化とMAX7219マトリクスディスプレイの設定
- `src/utils/bluetooth.rs` - BLEサーバー設定とCharacteristic管理

### モードシステム（計画中）
BLE経由でモード切替が可能。各モードでボタンの役割とLED表示が変わる。
- デジタルペット: アイドルアニメーション + ボタンインタラクション
- ポモドーロタイマー: カウントダウン表示
- ツール: サイコロ、YES/NO等
- 通知インジケーター: Wi-Fi経由で天気・通知表示
- スマートホーム: MQTT/HTTP連携
- PCモニター: CPU使用率・ミュート状態表示

### データフロー
1. BLE Characteristicにクライアントからコマンド/データを書き込み
2. コマンドに応じてモード切替またはデータ更新
3. データはNVS（不揮発性ストレージ）に永続化
4. メインループで現在のモードに応じたLED表示を更新

### BLE設定
- サービスUUID: `455aa9f0-2999-43de-81b4-54e0de255927`
- Characteristic UUID: `681285a6-247f-48c6-80ad-68c3dce18585`
- デバイス名: "LED BOX"
- パスキー: 123456
