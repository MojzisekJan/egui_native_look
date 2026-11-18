# Audit egui_native_look před vydáním

Datum: 18. listopadu 2026. Auditovaná verze: 0.3.0, commit `d48cc7bdf4d412f6bf2793012e613384de90e168`.
Repozitář: https://github.com/MojzisekJan/egui_native_look.
Lokální klon: `/home/jan/rust/egui_native_look`.

Klon byl před auditem čistý. `git pull --ff-only` potvrdil shodu s `origin/main`. Audit neprovádí opravy zdrojového kódu ani aktualizace závislostí. Přidává tuto zprávu a reprodukční testy.

Následné opravy všech tří chyb a nové ověření popisuje [FOLLOWUP.md](FOLLOWUP.md). Tato zpráva zachovává původní stav auditu.

## Závěr

Před vydáním doporučuji opravit tři reprodukované chyby níže. Nejdůležitější je tichá změna celočíselných dat pouhým vykreslením slideru. Běžné testy tyto situace nepokrývají. Úspěšný překlad a existující testy proto samy nestačí jako podklad k vydání.

Vlastní knihovna obsahuje dva bloky `unsafe`, oba ve Windows čtení registru. Kontrola jejich ukazatelů, délek a životnosti neodhalila chybu paměťové bezpečnosti. Audit nepotvrdil čtení nebo zápis mimo pole v knihovně. To není důkaz absence všech chyb ani bezpečnostní certifikace.

## Potvrzené nálezy

### 1. P1: Slider při vykreslení tiše poškodí velké celé číslo

Místo: `src/widgets/choice.rs:1794–1804`, zejména přiřazení na řádku 1802.

`Slider::controls` při `SliderClamping::Always` převádí původní `T` na `f64` a zpět při každém vykreslení, i když hodnota leží uvnitř rozsahu. Pro `u64` a `i64` nad přesností `f64` není převod bezeztrátový. Kontrola změny potom porovnává znovu jen `f64`, a tak skutečnou změnu modelu nepozná.

Reprodukce bez událostí uživatele:

```rust
let mut value = 9_007_199_254_740_993u64;
let response = ui.add(
    egui_native_look::Slider::new(&mut value, 0..=u64::MAX)
        .clamping(egui::SliderClamping::Always),
);
// Skutečnost: value == 9_007_199_254_740_992, response.changed() == false.
```

Dopad: aplikace může uložit změněná data bez toho, aby uživatel ovládací prvek použil. Kód sledující `changed()` se o změně nedozví. Větev je společná všem vzhledům.

Doporučená oprava: nepřepisovat hodnotu uvnitř rozsahu konverzí přes `f64`; při skutečném omezení nastavit přesný typovaný kraj rozsahu. Uchovat typované meze a porovnávat skutečnou hodnotu `T`. Přidat testy kolem `2^53`, pro kladné i záporné hodnoty a krajní `u64`/`i64`. Prověřit stejný vzorec také v číselných editorech a v detekci změn.

Test: `wide_integer_must_not_change_without_input`. Selhání bylo reprodukováno ve všech šesti vzhledech, v každém s `changed() == false`.

### 2. P2: Focus ring přeteče na platném poloměru

Místo: `src/paint.rs:170–175`.

`CornerRadius` má složky `u8`. Výpočet `radius.nw + 2` a ostatních rohů přeteče pro 254 a 255. Kód počítá zvětšený poloměr před výběrem vzhledu, takže může panikařit i vzhled, který ho dále nepotřebuje. Veřejné `focus_ring` přijímá takovou hodnotu přímo; `Metrics::radius` ji také nezakazuje.

Reprodukce během UI frame:

```rust
egui_native_look::focus_ring(
    ui.painter(),
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(100.0, 40.0)),
    egui::CornerRadius::same(255),
);
```

Výsledek v testovacím profilu: panic `attempt to add with overflow` na řádku 171. S vypnutou kontrolou přetečení se hodnota přetočí, takže geometrie bude chybná. Nejde o zápis mimo paměť.

Doporučená oprava: použít `saturating_add(2)` pro každý roh. Ověřit asymetrické rohy i hodnoty 253, 254 a 255 a běžný fokus widgetu po instalaci vlastních metrik.

Test: `maximum_corner_radius_does_not_panic`.

### 3. P2: Navigace IME kandidáty mění hodnotu GTK NumberField

Místo: `src/widgets/entry.rs:1396–1426`, zejména volání `number_input` na řádku 1424.

`gtk_number` zjistí aktivní kompozici a blokuje Enter i předběžné potvrzení textu. Přesto zavolá `number_input` bez podmínky `!composing`. Šipky nahoru/dolů a další číselné zkratky tak změní model a spotřebují událost i během kompozice.

Reprodukce: vykreslit GTK NumberField s hodnotou 5, nastavit mu fokus, poslat `ImeEvent::Preedit` s neprázdným textem a `ArrowUp`. Skutečná hodnota je 6, očekávaná zůstává 5. Navigace ve vstupní metodě nemá být číselným krokem.

Dopad: uživatel zadávající text pomocí IME může nechtěně změnit číslo ještě před potvrzením vstupu.

Doporučená oprava: během kompozice neprovádět číselné klávesové zkratky. Samostatně rozhodnout o přístupnostních požadavcích a kolečku. Ověřit také PageUp/Down, Ctrl+Home/End a pořadí commit/klávesové události.

Test: `numeric_ime_arrow_must_not_change_model`.

## Závislosti a vydávání

`cargo audit` proti aktuální RustSec databázi vrací neúspěch. Zamčený strom obsahuje:

- Tokio 1.54.0, `RUSTSEC-2026-0384`, opraveno od 1.54.1. Advisory popisuje race/use-after-free v Interval na vícevláknovém runtime.
- rustix 1.1.5, `RUSTSEC-2026-0486`, opraveno od 1.1.6. Advisory popisuje nesprávnou inicializaci paměti u xattr funkcí s nulovým bufferem.
- Upozornění na neudržované `zbus_names`, `zvariant` a `zvariant_derive`.
- Yanked verze `async-executor` 1.14.0, `clipboard-win` 5.4.1 a Tokio 1.54.0.

Tokio i rustix se sem dostávají přes vývojové závislosti. Běžný strom knihovny je neobsahuje. Tokio má v prohlédnutém sjednoceném stromu features `rt,time`, nikoli `rt-multi-thread`. Audit tedy neprokazuje dosažitelné use-after-free v této knihovně nebo galerii. Podobně nebyla prokázána cesta k postiženým xattr funkcím rustix. Před vydáním přesto obnovit postižené položky lockfile a znovu spustit testy. Informativní upozornění řešit přes aktualizaci nadřazených závislostí, nikoli plošným ignorováním advisories. Přesná data jsou v `logs/rustsec.json`.

Jediný nalezený GitHub workflow `.github/workflows/pages.yml` staví a publikuje galerii. Nespouští audit, interakční testy, Clippy ani kontrolu MSRV při pull requestu. Doporučuji samostatný CI workflow pro release kontroly popsané v README, včetně obou konfigurací features a kontroly platforem. Opravy tří nálezů přidat do hlavní regresní sady.

## Další prověřené oblasti

Kontrola zahrnovala všech 17 souborů knihovny a prohlídku testů, manifestu, galerie, benchmarku, webového obalu a workflow. Podrobněji byly prověřeny numerické vstupy, indexování voleb a seznamů, UTF-8 a znakové indexy editoru, IME, zakázané ovládací prvky, modalita, fokus, systémové procesy, parsování barev a práce s fonty.

- Indexy uživatelského výběru obvykle používají `get` nebo se před navigací omezují. Prázdné kolekce mají ochrany. Indexování Windows accent palety pracuje uvnitř inicializovaného 32bytového bufferu a kontroluje dostatečnou délku výsledku.
- Řezání UTF-8 editoru vychází z `char_indices`; nepoužívá vstupní znakový index přímo jako byte offset. CSS parser kontroluje počet složek před indexováním a omezuje rekurzi aliasů na 16.
- Externí procesy používají `Command::args`, nikoli shell. Název GTK tématu odmítá oddělovače cesty a `.`/`..`. Nebyla nalezena shell injection.
- Systémová volání `gsettings`, `fc-match` a `defaults` nemají timeout; `.output()` čeká na proces a sbírá celý výstup. Zaseknutý proces může blokovat setup a v background followeru zanechat trvale čekající požadavek. Doporučení pro odolnost: deadline, ukončení a reap procesu, limit výstupu a fallback. Nejde o prokázané vzdálené spuštění kódu.
- Heslová pole předávají režim password egui a vypínají Cut/Copy v kontextovém menu. Existující testy kontrolují kopírování, zakázané přístupnostní editace a Unicode.

## Ověření

Výsledky kontrol jsou doplněny níže. Reprodukční testy v `repro` jsou záměrně formulované jako požadované správné chování, takže na auditovaném commitu selhávají. Spuštění z tohoto adresáře:

```sh
cargo test --manifest-path repro/Cargo.toml
```

Reprodukce používají veřejné API a nevkládají žádné změny do knihovny. Lockfile reprodukcí vychází z lockfile projektu. Při prvním spuštění byl použit samostatný adresář v `/tmp`, proto zaznamenaný log může ukazovat původní absolutní cestu.

Testy na Linuxu a cross-compilation nenahrazují běh na Windows/macOS ani fyzických iOS/Android zařízeních. Nebyl proveden dlouhodobý fuzzing, Miri, dynamická analýza nativních platformních API ani úplný audit zdrojů všech tranzitivních závislostí. Zpráva odděluje reprodukované chyby od doporučení pro další zpevnění.

### Výsledky kontrol při auditu

Prostředí: Linux x86_64, rustc/cargo 1.100.0. Samostatná kontrola MSRV použila rustc 1.95.0.

| Kontrola | Výsledek |
| --- | --- |
| `cargo fmt --all -- --check` | Prošlo |
| `cargo clippy --locked --all-targets --all-features -- -D warnings` | Prošlo |
| `cargo clippy --locked --all-targets --no-default-features -- -D warnings` | Prošlo |
| `cargo test --all-targets --locked` | 270 unit/integračních testů + 8 testů galerie prošlo; 4 snapshoty standardně ignorovány; 7 benchmark smoke případů prošlo |
| `cargo test --locked --no-default-features` | 267 unit/integračních + 49 dokumentačních testů prošlo; 4 snapshoty standardně ignorovány |
| `cargo test --locked --test snapshots -- --include-ignored` | Všechny 4 vizuální testy prošly, reference nebyly přepisovány |
| `cargo +1.95.0 check --lib --locked` se samostatným target-dir | Prošlo |
| Node 22 `--test web/viewport.test.mjs` | Prošlo |
| `cargo package --list --locked` | Prošlo; docs/web v publikačním seznamu nejsou |
| `cargo audit --json` | Neprošlo, advisories a upozornění popsané výše |
| Vlastní 3 regresní reprodukce | Všechny 3 správně odhalily chybu, tedy selhaly |

Při kontrole balíčku byl ověřen seznam souborů, nikoli úplné ověření publikace na crates.io. Knihovna nebyla publikována a do GitHubu nebylo nic odesláno.

Cross-compilation `cargo check --lib --locked --target <target>` prošla pro všech pět targetů: `wasm32-unknown-unknown`, `x86_64-pc-windows-gnu`, `aarch64-apple-darwin`, `aarch64-apple-ios` a `aarch64-linux-android`. Následná kontrola knihovny bez default features také prošla. Samostatné `cargo test --doc --locked` potvrdilo všech 49 dokumentačních testů s výchozími features.
