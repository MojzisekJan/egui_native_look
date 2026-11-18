# Opravy nálezů a opakované ověření

Datum: 18. listopadu 2026. Navazuje na audit commitu `d48cc7b`, verze 0.3.0. Změny zůstávají v pracovním stromu, verze balíčku nebyla zvýšena.

Všechny tři funkční chyby z auditu jsou opravené. Regresní testy jsou součástí hlavní sady.

Další úpravy timeoutů, přesného interaktivního zadávání a běhové kontroly popisuje [HARDENING.md](HARDENING.md).

## Opravy

- Slider uchovává původní typované meze. Při `Always` clamping kontroluje a případně omezuje skutečné `T`, takže se velká celá čísla uvnitř rozsahu nepřepisují přes `f64`. Skutečné omezení na přesnou mez vrací `changed()`, i když původní i nová hodnota mají stejné vyjádření ve `f64`.
- Focus ring zvětšuje všechny rohy saturujícím sčítáním. Hodnoty 254 a 255 nepřetečou a nezpůsobují panic.
- GTK číselné pole během IME kompozice ani ve frame s commit událostí neprovádí numerické zkratky. Kandidátní navigace neprochází ani do TextEdit, kde by mohla zrušit preedit a zdvojit potvrzený text. Po potvrzení se numerické ovládání obnoví.

## Regresní testy

- `tests/controls.rs::always_clamped_sliders_preserve_wide_integers_and_exact_bounds`: všech šest vzhledů, hodnoty kolem `2^53`, krajní `u64`/`i64`, obrácené i sousedící meze, zachování hodnot bez vstupu a správné hlášení skutečného omezení. Ověřuje i variantu s editorem hodnoty.
- `tests/controls.rs::focus_rings_accept_large_and_asymmetric_corner_radii`: rohy 253–255 a asymetrické rohy, všech šest vzhledů, fokus běžného tlačítka s vlastními metrikami a skutečná saturace vykresleného Windows ringu na 255.
- `tests/gtk_reference.rs::gtk_numeric_ime_navigation_preserves_model_and_draft`: šipky, Page Up/Down, Ctrl+Home/End i Ctrl+Page Up/Down v obyčejném poli a formuláři. Ověřuje zachování modelu, draftu i fokusu během preedit, průběžné kompozice a commit frame a následný Enter a číselný krok.

## Aktualizace závislostí

| Balíček | Před | Po |
| --- | --- | --- |
| Tokio | 1.54.0 | 1.54.1 |
| rustix | 1.1.5 | 1.1.6 |
| async-executor | 1.14.0 | 1.14.1 |
| clipboard-win | 5.4.1 | 5.4.3 |

Aktualizovány jsou také kompatibilní patch verze AccessKit a jeho adaptérů: accesskit 0.25.2, accesskit_consumer 0.39.2, accesskit_atspi_common 0.21.1, accesskit_unix 0.24.1, accesskit_winit 0.34.2, accesskit_ios 0.2.2, accesskit_macos 0.27.2 a accesskit_windows 0.35.2. Manifest a veřejná kompatibilita s egui 0.37 se nemění.

Aktuální `cargo audit` pro hlavní lockfile hlásí **0 zranitelností**, žádné `unsound` ani `yanked` upozornění a vrací exit code 0. Zůstávají tři informační upozornění `unmaintained` pro zvariant, zvariant_derive a zbus_names. Jsou ve vývojovém stromu přes AccessKit/AT-SPI a zbus 5.19. I nejnovější kompatibilní accesskit_unix 0.24.1 stále vyžaduje zbus 5.19; udržovaná náhrada těchto balíčků je součástí zbus 6. Prostá kompatibilní aktualizace lockfile tuto migraci neumožňuje. Upozornění nebyla potlačena a přístupnost galerie zůstává zapnutá.

## Ověření konečných změn

Všechny kontroly v následující tabulce prošly. Při rozšíření IME testu se nejprve odhalilo zdvojení preedit textu; tabulka a přiložené logy obsahují až následné opakované testy po dokončení opravy.

| Kontrola | Výsledek |
| --- | --- |
| `cargo fmt --all -- --check` | Prošlo, [log](fixes/format-final.log) |
| `cargo test --locked --all-features --all-targets` | Prošlo, [log](fixes/test-all-final.log) |
| `cargo test --locked --doc --all-features` | Prošlo, [log](fixes/test-doc.log) |
| `cargo test --locked --no-default-features` | Prošlo, [log](fixes/test-min-final.log) |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Prošlo, [log](fixes/clippy-all.log) |
| `cargo clippy --locked --all-targets --no-default-features -- -D warnings` | Prošlo, [log](fixes/clippy-min.log) |
| `cargo test --locked --test snapshots -- --include-ignored` | Prošlo, [log](fixes/snapshots.log) |
| `cargo +1.95.0 check --lib --locked --target-dir /tmp/egui-native-msrv` | Prošlo, [log](fixes/msrv.log) |
| `cargo check --lib --locked --target wasm32-unknown-unknown` | Prošlo, [log](fixes/wasm32-unknown-unknown.log) |
| `cargo check --lib --locked --target x86_64-pc-windows-gnu` | Prošlo, [log](fixes/x86_64-pc-windows-gnu.log) |
| `cargo check --lib --locked --target aarch64-apple-darwin` | Prošlo, [log](fixes/aarch64-apple-darwin.log) |
| `cargo check --lib --locked --target aarch64-apple-ios` | Prošlo, [log](fixes/aarch64-apple-ios.log) |
| `cargo check --lib --locked --target aarch64-linux-android` | Prošlo, [log](fixes/aarch64-linux-android.log) |
| `cargo check --locked --example gallery --target wasm32-unknown-unknown` | Prošlo, [log](fixes/wasm-gallery.log) |
| `/home/jan/.local/share/pi-node/node-v22.23.3-linux-x64/bin/node --test web/viewport.test.mjs` | Prošlo, [log](fixes/web-viewport.log) |
| `cargo audit --json` | Prošlo, [log](fixes/audit.log) |
| `cargo package --list --locked --allow-dirty` | Prošlo, [log](fixes/package-list.log) |
| `git diff --check` | Prošlo, [log](fixes/diff-check.log) |

Přesné příkazy a návratové kódy jsou v [results.json](fixes/results.json). Kontrola balíčku ověřuje seznam souborů, nikoli publikaci. Vizuální testy používají původní reference, žádný snapshot nebyl přepsán. Nativní platformy byly ověřeny cross-compilation; fyzické Windows/macOS/iOS/Android prostředí nebylo během této opravy k dispozici. Převody pro polohu railu a samotnou interaktivní aritmetiku nadále používají egui `Numeric`/`f64`; tato oprava garantuje přesné zachování a omezení existující hodnoty, nikoli jednotkovou přesnost interaktivního pohybu v libovolném 64bitovém rozsahu.

Do GitHubu ani na crates.io nebylo nic odesláno.
