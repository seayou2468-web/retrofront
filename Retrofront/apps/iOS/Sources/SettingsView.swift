import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var runtime: EmulatorRuntimeModel

    var body: some View {
        AppScreen(title: "Settings", subtitle: "実際に保存・反映されるアプリ設定") {
            SettingsGroup(title: "Video") {
                SettingPickerRow(title: "Renderer", subtitle: "Software / Metal / MoltenVK / OpenGL ES", value: runtime.rendererLabel, choices: runtime.rendererChoices) { choice in
                    runtime.setRenderer(choice.value)
                }
                SettingPickerRow(title: "Scale", subtitle: "画面比率の扱い", value: runtime.videoScaleModeLabel, choices: runtime.videoScaleModeChoices) { choice in
                    runtime.setVideoScaleMode(choice.value)
                }
                SettingPickerRow(title: "Filter", subtitle: "ピクセル描画品質", value: runtime.videoFilterLabel, choices: runtime.videoFilterChoices) { choice in
                    runtime.setVideoFilter(choice.value)
                }
                SettingToggleRow(title: "VSync", subtitle: "表示更新に同期", isOn: runtime.vsyncEnabled) {
                    runtime.setVsyncEnabled($0)
                }
            }

            SettingsGroup(title: "Audio") {
                SettingToggleRow(title: "Audio Output", subtitle: "音声出力を有効化", isOn: runtime.audioEnabledSetting) {
                    runtime.setAudioEnabled($0)
                }
                SettingToggleRow(title: "Audio Sync", subtitle: "音声同期", isOn: runtime.audioSyncSetting) {
                    runtime.setAudioSync($0)
                }
                SettingPickerRow(title: "Latency", subtitle: "出力遅延", value: runtime.audioLatencyLabel, choices: runtime.audioLatencyChoices) { choice in
                    runtime.setAudioLatency(choice.value)
                }
            }

            SettingsGroup(title: "Menu") {
                SettingPickerRow(title: "Menu Driver", subtitle: "Ozone / XMB / RGUI / Material UI", value: runtime.menuDriverLabel, choices: runtime.menuDriverChoices) { choice in
                    runtime.setMenuDriver(choice.value)
                }
            }

            SettingsGroup(title: "Controller") {
                SettingToggleRow(title: "Touch Overlay", subtitle: "画面上コントローラー", isOn: runtime.overlayEnabledSetting) {
                    runtime.setOverlayEnabledSetting($0)
                }
                SettingPickerRow(title: "Overlay Set", subtitle: "RetroArch overlay .cfg", value: runtime.overlaySelectionLabel, choices: runtime.availableOverlays.map { (label: $0.label, value: $0.path) }) { choice in
                    if let overlay = runtime.availableOverlays.first(where: { $0.path == choice.value }) { runtime.selectOverlay(overlay) }
                }
                SettingPickerRow(title: "Overlay Opacity", subtitle: "タッチ操作の透明度", value: runtime.overlayOpacityLabel, choices: runtime.overlayOpacityChoices) { choice in
                    runtime.setOverlayOpacity(choice.value)
                }
                SettingToggleRow(title: "Haptics", subtitle: "タッチ操作の振動フィードバック", isOn: runtime.hapticsEnabledSetting) {
                    runtime.setHapticsEnabled($0)
                }
            }

            SettingsGroup(title: "Library") {
                SettingPickerRow(title: "Sort", subtitle: "ROMの並び順", value: runtime.librarySortLabel, choices: runtime.librarySortChoices) { choice in
                    runtime.setLibrarySort(choice.value)
                }
                SettingToggleRow(title: "Core Badges", subtitle: "互換コア数をROMに表示", isOn: runtime.libraryCoreBadgesEnabled) {
                    runtime.setLibraryCoreBadgesEnabled($0)
                }
                SettingToggleRow(title: "File Details", subtitle: "拡張子とサイズを表示", isOn: runtime.libraryFileDetailsEnabled) {
                    runtime.setLibraryFileDetailsEnabled($0)
                }
                SettingToggleRow(title: "Auto Scan", subtitle: "起動時にROMを再スキャン", isOn: runtime.libraryAutoScanEnabled) {
                    runtime.setLibraryAutoScanEnabled($0)
                }
            }

            SettingsGroup(title: "Loaded Core") {
                SettingInfoRow(title: "Current Core", value: runtime.systemInfo?.libraryName ?? runtime.corePath ?? "Not loaded")
                if runtime.availableCores.isEmpty {
                    SettingInfoRow(title: "Bundled Cores", value: "No bundled cores discovered")
                } else {
                    ForEach(runtime.availableCores, id: \.path) { core in
                        Button {
                            runtime.loadBundledCore(core)
                        } label: {
                            CoreRow(core: core)
                        }
                        .buttonStyle(.plain)
                    }
                }
                if runtime.coreOptions.isEmpty {
                    SettingInfoRow(title: "Core Options", value: "Load a core to edit its options")
                } else {
                    ForEach(runtime.coreOptions, id: \.key) { option in
                        SettingPickerRow(title: option.desc.isEmpty ? option.key : option.desc, subtitle: option.info.isEmpty ? option.key : option.info, value: option.value, choices: option.values.map { (label: $0.label.isEmpty ? $0.value : $0.label, value: $0.value) }) { choice in
                            runtime.setOption(key: option.key, value: choice.value)
                        }
                    }
                }
            }

            SettingsGroup(title: "Storage") {
                SettingInfoRow(title: "Content Folder", value: runtime.settingValue("content_directory"))
                SettingInfoRow(title: "Core Folder", value: runtime.settingValue("libretro_directory"))
                SettingInfoRow(title: "Info Folder", value: runtime.settingValue("libretro_info_path"))
                SettingInfoRow(title: "Saves", value: runtime.settingValue("savefile_directory"))
                SettingInfoRow(title: "States", value: runtime.settingValue("savestate_directory"))
                SettingInfoRow(title: "System/BIOS", value: runtime.settingValue("system_directory"))
                SettingInfoRow(title: "Screenshots", value: runtime.settingValue("screenshot_directory"))
                ForEach(FrontendAssetArchive.allCases) { archive in
                    HStack(spacing: 10) {
                        Button {
                            runtime.installBundledAsset(archive)
                        } label: {
                            Label("Install bundled \(archive.displayName)", systemImage: "archivebox.fill")
                                .font(.subheadline.bold())
                                .foregroundColor(RetroArchMenuPalette.driver("materialui").accent)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .padding(12)
                                .background(RetroArchMenuPalette.driver("materialui").elevated)
                                .clipShape(RoundedRectangle(cornerRadius: RetroArchMenuMetrics.compactRadius, style: .continuous))
                        }
                        .buttonStyle(.plain)

                        Button {
                            runtime.downloadAndInstallAsset(archive)
                        } label: {
                            Label("Fetch", systemImage: "icloud.and.arrow.down.fill")
                                .font(.subheadline.bold())
                                .foregroundColor(RetroArchMenuPalette.teal)
                                .padding(12)
                                .background(RetroArchMenuPalette.driver("materialui").elevated)
                                .clipShape(RoundedRectangle(cornerRadius: RetroArchMenuMetrics.compactRadius, style: .continuous))
                        }
                        .buttonStyle(.plain)
                    }
                }
            }
        }
    }
}

