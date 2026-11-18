# Další zpevnění před vydáním

Navazuje na [opravy tří nálezů](FOLLOWUP.md). Datum: 18. listopadu 2026. Změny CI jsou mimo tento požadavek.

## Systémové procesy

Všechna volání `gsettings`, `fc-match` a `defaults` používají společný helper v `src/system/process.rs`:

- Jeden dotaz má deadline 500 ms. Při timeoutu, chybě dotazu nebo překročení limitu pomocník ukončí přímý podproces a vyčká na jeho ukončení.
- Dotaz přijímá nejvýše 64 KiB stdout. Stdin a stderr směřují na null; výstup se čte omezeně.
- Soukromý dočasný soubor vzniká atomickým `create_new` s Unix oprávněními 0600. Limit velikosti se kontroluje během běhu; přijatá data mají pevný limit. Soubor se uklízí i při chybě. Průběžně zapsaný objem může mezi kontrolami krátce překročit limit.
- Čtení nečeká na EOF zděděné pipe a nepoužívá pomocná čtecí vlákna. Potomek držící zděděný stdout tak nezablokuje volající aplikaci.
- Zachováno je macOS rozlišení světlého režimu, při němž `defaults` vrací nenulový status. Timeout nebo nemožnost spuštění vrací `None`.

Testy reálně spouštějí procesy a ověřují úspěch, nenulový status, nadlimitní výstup, timeout a zděděný stdout. Dotazy při inicializaci zůstávají synchronní, ale čekání na podproces je omezené; systémový follower je nadále provádí mimo UI vlákno.

## Přesné interaktivní zadávání

`src/numeric.rs` poskytuje bezpečné rozlišení builtin integer typů přes `Any`, kontrolované převody a aritmetiku v `i128`. Podporuje builtin integer typy do 64 bitů a odpovídající unsigned NonZero typy do 64 bitů. Nová cesta nepřidává `unsafe` ani produkční závislosti.

- Keyboard, kolečko, steppery a přístupnostní increment/decrement mohou měnit velká čísla o přesnou jednotku a hlásí skutečnou změnu `T`.
- Textové editory používají přesný desetinný zápis a parser. Zachovají například `9007199254740993` i `18446744073709551615` a správně zachytí externí změnu modelu mezi čísly se stejným vyjádřením ve `f64`.
- Pro široké integer rozsahy používají všechny vzhledy textový editor místo `DragValue` scrubbingu. Zůstávají zachované prefixy, suffixy, potvrzování a IME ochrany.
- Krátký sliderový rozsah u velkých čísel počítá pointerovou pozici z rozdílu integer mezí. Shodné `f64` reprezentace sousedních mezí ovládání nevypnou.
- Přesné jsou i koncové body, obrácené rozsahy, krokování logaritmického slideru a přístupnostní textové SetValue. Těsné logaritmické intervaly použijí relativní lineární výpočet, aby se předešlo nulovému rozdílu zaokrouhlených logaritmů.
- `SliderClamping::Never` zachovává volný textový editor v rozsahu daného integer typu a odpovídající přístupnostní metadata.

Pointer má konečný počet rozlišitelných poloh; pro výběr každé jednotlivé hodnoty v celém 64bitovém rozsahu jsou určeny klávesy a text. Uživatelské callbacky s podpisem `f64` a přístupnostní NumericValue nadále nesou přesnost svého typu. Přístupnostní textová hodnota je přesná. Vlastní implementace `Numeric` a NonZeroU128 používají dosavadní fallback přes `f64`; přesná cesta je dokumentována pro podporované typy do 64 bitů.

Regresní pokrytí zahrnuje devět interakčních testů v `tests/precision.rs`, dva testy převodů/parseru/aritmetiky a tři testy systémových procesů. Interakce pokrývají všechny vzhledy, `u64::MAX`, `i64::MIN`, `i64::MAX`, hranici `2^53`, pointer, klávesy, text, přístupnost a zachování kolečka ostatních vzhledů.

## Závěrečné ověření

Všech 18 kontrol v tabulce prošlo nad dokončeným kódem. Vizuální reference nebyly přepisovány.

| Kontrola | Výsledek |
| --- | --- |
| `cargo fmt --all -- --check` | Prošlo, [log](hardening/format.log) |
| `cargo test --locked --all-features --all-targets` | Prošlo, [log](hardening/test-all.log) |
| `cargo test --locked --doc --all-features` | Prošlo, [log](hardening/test-doc.log) |
| `cargo test --locked --no-default-features` | Prošlo, [log](hardening/test-min.log) |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Prošlo, [log](hardening/clippy-all.log) |
| `cargo clippy --locked --all-targets --no-default-features -- -D warnings` | Prošlo, [log](hardening/clippy-min.log) |
| `cargo test --locked --test snapshots -- --include-ignored` | Prošlo, [log](hardening/snapshots.log) |
| `cargo +1.95.0 check --lib --locked --target-dir /tmp/egui-native-msrv` | Prošlo, [log](hardening/msrv.log) |
| `cargo check --lib --locked --target wasm32-unknown-unknown` | Prošlo, [log](hardening/wasm32-unknown-unknown.log) |
| `cargo check --lib --locked --target x86_64-pc-windows-gnu` | Prošlo, [log](hardening/x86_64-pc-windows-gnu.log) |
| `cargo check --lib --locked --target aarch64-apple-darwin` | Prošlo, [log](hardening/aarch64-apple-darwin.log) |
| `cargo check --lib --locked --target aarch64-apple-ios` | Prošlo, [log](hardening/aarch64-apple-ios.log) |
| `cargo check --lib --locked --target aarch64-linux-android` | Prošlo, [log](hardening/aarch64-linux-android.log) |
| `cargo check --locked --example gallery --target wasm32-unknown-unknown` | Prošlo, [log](hardening/wasm-gallery.log) |
| `/home/jan/.local/share/pi-node/node-v22.23.3-linux-x64/bin/node --test web/viewport.test.mjs` | Prošlo, [log](hardening/web-viewport.log) |
| `cargo audit --json` | Prošlo, [log](hardening/audit.log) |
| `cargo package --list --locked --allow-dirty` | Prošlo, [log](hardening/package-list.log) |
| `git diff --check` | Prošlo, [log](hardening/diff-check.log) |

RustSec vrací 0 zranitelností, bez `unsound` a `yanked` upozornění. Zůstávají dříve popsaná tři informační `unmaintained` upozornění ve vývojovém stromu zbus 5. Přesné příkazy a návratové kódy jsou v [results.json](hardening/results.json).

## Skutečné běhové ověření

Na Linuxu proběhlo sedm úspěšných startů nativní eframe galerie přes její běžný window/renderer backend. Každý běh vykreslil UI, uložil vlastní screenshot a ukončil se s kódem 0. Screenshot GTK light byl vizuálně zkontrolován.

| Vzhled a schéma | Podklady |
| --- | --- |
| gtk-light | [Screenshot](hardening/runtime/gtk-light.png), [log](hardening/runtime/gtk-light.log) |
| gtk-dark | [Screenshot](hardening/runtime/gtk-dark.png), [log](hardening/runtime/gtk-dark.log) |
| mac-light | [Screenshot](hardening/runtime/mac-light.png), [log](hardening/runtime/mac-light.log) |
| windows-light | [Screenshot](hardening/runtime/windows-light.png), [log](hardening/runtime/windows-light.log) |
| ios-light | [Screenshot](hardening/runtime/ios-light.png), [log](hardening/runtime/ios-light.log) |
| material-light | [Screenshot](hardening/runtime/material-light.png), [log](hardening/runtime/material-light.log) |
| plain-light | [Screenshot](hardening/runtime/plain-light.png), [log](hardening/runtime/plain-light.log) |

Všechny tyto běhy proběhly na Linuxu. Běhové ověření na Windows, macOS, iOS a Android je podle následné dohody s uživatelem odloženo na později. Android SDK je dostupné, ale [adb devices](hardening/runtime/android-devices.log) nehlásí žádné připojené zařízení. Pro tyto platformy prošla cross-compilation knihovny; reálná mobilní klávesnice, IME a platformní přístupnost vyžadují další běh na jejich systému. In-memory IME a touch regrese jsou součástí automatických testů.

Změny zůstávají v pracovním stromu a nejsou publikované.
