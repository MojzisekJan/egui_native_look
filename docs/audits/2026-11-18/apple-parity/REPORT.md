# egui_native_look proti SwiftUI — macOS a iOS

Výchozí revize: `a8d5ce9`. Audit a lokální opravy z 18. 11. 2026.

**Závěr: knihovna se základními tvary blíží SwiftUI, ale není jeho funkční ani
vizuální náhradou 1:1.** Část rozdílů byla opravena a podložena nezávislou nativní
referencí. Největší zbývající problémy jsou kontrakty výchozí akce a fokusu,
velikosti různých iOS ovládacích prvků a nativní textové interakce. Vlastní egui
snapshoty samy o sobě shodu se SwiftUI nedokazují.

## Co se skutečně porovnávalo

- macOS 27.0.1, build 26A434, Apple silicon; skutečné SwiftUI v `NSHostingView`.
- iOS 26.5, iPhone 17 v existujícím simulátoru, měřítko 3×; SwiftUI v `UIHostingController`.
- egui 0.37.0 a místní knihovna, render přes Metal na Macu, 2×.
- Dvanáct nativních variant: světlá/tmavá, povolená/zakázaná, malá a velká velikost.
- Nativní macOS události pro Return, Command+Return, Shift+Return, kliknutí,
  opuštění tlačítka a návrat před uvolněním, a Return z textového pole.
- Rust knihovna přilinkovaná do skutečného procesu iOS simulátoru ověřila
  načtení systémového písma a rozměry dotykových cílů.

Nativní reference nečte rozměry ani barvy z knihovny. macOS se spouští jako
řádný app bundle a každý snímek musí potvrdit aktivní aplikaci i klíčové okno;
původní snímky neaktivního okna byly při kontrole odmítnuty a nahrazeny. Snímky ze simulátoru nesou
barevný profil; před srovnáním jsou převedeny do sRGB. Geometrie je v logických
bodech, nikoli v pixelech. Odlišná rasterizace písma a měřítko snímání vylučují
poctivé tvrzení o jednom celkovém procentu shody.

## Opravené rozdíly

| Oblast | Před opravou | Nativní reference / výsledná změna |
| --- | --- | --- |
| iOS systémové písmo | `system_fonts(Ios)` vracelo `None` | CoreText vyhledá skutečný UI font; v simulátoru ověřeno načtení a sazba. Na Macu iOS náhled používá místní SF. Fonty se nepřibalují. |
| macOS textová pole | small 23, regular/large 26 bodů | SwiftUI: 21 / 24 / 24. Stejně upraven i sdílený základ číselných polí. |
| macOS switch | small 45, large 63 bodů na šířku | SwiftUI: 44×20 / 54×24 / 64×28. |
| macOS stisk tlačítka | Opuštění oblasti zrušilo klik i po návratu | Nativní události potvrzují aktivaci při návratu a uvolnění uvnitř. Opraveno; dotyk, ztráta fokusu a zakázání stále ruší gesto. |
| iOS switch | 63bodová kapsle a 2 body vnějšího prostoru | SwiftUI 61×28 bez boční mezery; odstraněn nenativní stín knoflíku. |
| iOS segmenty | 34 bodů, při dotyku se zvětšovala i kresba | Běžná kapsle 31 bodů; větší dotykový cíl je oddělen od kresby a jeho okraje zůstávají aktivní. |
| iOS vybraný segment | Stejná váha textu, přidaný stín | Semibold vybrané položky a kapsle bez stínu. |
| Apple tlačítka | Výchozí akce bez typografického důrazu; destruktivní vždy červená výplň | Výchozí akce používá důrazové písmo. Běžné macOS destruktivní tlačítko zůstává neutrální; iOS používá červený text. Explicitní vlastní bold metriky mají přednost. |
| Zakázaný iOS slider | Šedá aktivní část a bílý knoflík v tmavém režimu | Zachovaný akcent zeslabený na polovinu, knoflík a stín rovněž zeslabené; tmavá neaktivní kolej opravena. |
| Zakázané iOS segmenty/switch | Nesprávné zeslabení jednotlivých částí | Upraven track, text a vybraný segment; knoflík switche v tmavém stavu šedý. |
| Zakázané iOS textové pole | Zeslabený obsah pole | SwiftUI zachovává čitelnost obsahu a mění podklad/okraj. Opraveno; pole stále není editovatelné. Okraj má jeden fyzický pixel. |

Původní test `mac_control_sizes_follow_swiftui` kontroloval nesprávné hodnoty
45/23 a 63/26, aniž by při testu běželo SwiftUI. Jeho očekávání nyní vycházejí
z nezávislé reference. Nový `tests/apple_reference.rs` pokrývá i návrat kurzoru,
zrušení dotyku, aktivaci v dotykovém okraji segmentu a zakázaný slider.

## Zbývající rozdíly podle priority

### P1 — výchozí akce a fokus nejsou kontraktem SwiftUI

Nativní reference potvrzuje, že Return aktivuje výchozí akci v běžném okně i
z editovaného pole; Command+Return ji neaktivuje, Shift+Return ano. V knihovně
`Button::default_action()` stále funguje pouze uvnitř `Dialog`. Zůstává také
oddělené nastavení akcentu a jiná politika při fokusu jiného tlačítka.

To není vhodné opravit pouhým `key_pressed(Enter)` u tlačítka: muselo by se
vyřešit pořadí vykreslování, modalita, výběr první akce v hierarchii, spotřebování
klávesy editorem, otevřené menu a IME. Potřebná je společná obsluha výchozí a
rušící akce na úrovni okna. Stávající bezpečnost IME se nesmí obětovat vzhledu.

### P1 — iOS controlSize není jeden násobek celého formuláře

Naměřeno: běžné tlačítko 34⅓ bodu, malé 28, velké 50⅓. Běžný i malý segmented
picker má 31 bodů, velký 48. Textové pole zůstává 34, switch 61×28 a slider 31
bodů bez ohledu na tyto velikosti. Současné `Metrics::small/large(Ios)` mění
sdílené metriky; velké tlačítko tak má 40 bodů. Pravidla je nutné rozdělit podle
typu prvku, včetně oddělení `controlSize`, velikosti textu a Dynamic Type.

### P1 — mobilní textové interakce a systémová integrace

Knihovna poskytuje egui kontextové menu a vlastní textové chování. To samo
nezajišťuje shodu s nativními iOS úchyty výběru, lupou, nabídkou úprav,
autokorekcí, diktováním, autofillem a chováním softwarové klávesnice.
Plná shoda těchto funkcí pravděpodobně vyžaduje rozhraní k nativnímu editoru,
nebo explicitně omezený a zdokumentovaný kontrakt čistého egui editoru.

V tomto auditu se neprováděl fyzický iPhone test, end-to-end přenos UIKit
dotykových/klávesových událostí do egui ani poslech VoiceOver. AccessKit testy
ověřují exportované uzly a akce; nenahrazují tyto systémové testy.

### P2 — rozložení, typografie a zbylá kresba

- SwiftUI `.frame(width:)` může vystředit vnitřní control s přirozenou šířkou;
  egui `.width()` nastavuje šířku samotného prvku. To vysvětluje část rozdílů
  pickeru a macOS segmentů v kontaktových listech. Srovnávací scénáře mají
  rozlišovat vnější rámec, vnitřní control a jeho zarovnání.
- Systémové písmo odstranilo zásadní rozdíl iOS, ale ještě nezaručuje totožnou
  sazbu, padding ani vyhlazení. Například běžné iOS tlačítko Save má v referenci
  šířku 61 bodů, aktualizované egui přibližně 58,47. Je třeba samostatně změřit
  vnitřní text a padding, ne kompenzovat rozdíl náhodným zvětšením všeho.
- Stín iOS slideru, některé odstíny zakázaných tlačítek a přesné odstíny
  segmentů stále nejsou totožné. Chybí nezávislá matice stisk/hover/fokus
  a více systémových akcentů. Nebyla ověřena Liquid Glass kompozice nad
  různými podklady ani Reduce Transparency/Reduce Motion.
- Dialogy, vnořená menu, toolbar, listy a scroll view nebyly pokryty stejně
  důkladnou nativní stavovou maticí jako základní prvky. Vlastní egui testy
  pro ně nadále existují, ale nejsou důkazem shody se SwiftUI.

## Co v obrázcích není porovnání stejného prvku

SwiftUI automatic Toggle na iOS je switch; explicitní egui Checkbox je jiný
control. `NumberField` obsahuje editor, zatímco nativní Stepper obsahuje popisek
a dvě akce. Tyto řádky jsou uvedeny pro odhalení rozdílu mapování, nikoli jako
pixelová chyba. Nativní snímek secure fieldu neobsahuje glyphy hesla, a proto
jej nepoužíváme k hodnocení maskování. Text stavového řádku se také lokalizuje
jinak. Srovnávací skript z těchto řádků neodvozuje celkové skóre shody.

## Ověření a reprodukce

Závěrečná kontrola finálních zdrojů prošla:

| Kontrola | Výsledek |
| --- | --- |
| Rust, všechny features | 302 jednotkových/funkčních + 49 dokumentačních testů |
| GPU snapshoty | 4 skupiny, 48 referenčních obrázků všech vzhledů |
| Rust bez výchozích features | 347 testů včetně dokumentace |
| Clippy na macOS a cíli iOS simulator | Bez varování při `-D warnings` |
| Formátování Rust, syntaxe Python, whitespace diffu | V pořádku |
| SwiftUI runner spuštěný přímo z repozitáře | 12 variant, 8 macOS vstupních asercí, iOS font probe |

Celkem standardní sada včetně dokumentace a GPU skupin obsahuje **355 úspěšných
testů**. Běh bez systémové integrace je další samostatná konfigurace.

Viz `checks/` a `verification.json` pro příkazy a úplné výsledky běhu;
`source-sha256.json` identifikuje zkontrolované zdroje. Reprodukovatelný runner,
zdroj SwiftUI reference, egui fixture a iOS Rust probe jsou v
`tools/apple-parity`. Zdrojové soubory a měření jsou uloženy spolu se snímky;
záměrně nejsou přiloženy systémové fonty ani build adresáře.

Při obrazových kontrolách byl u nezměněného GTK light menu na Metal zjištěn
jeden pixel nad původní tolerancí; největší rozdíl byl 2 hodnoty RGB kanálu.
Použita je dokumentovaná per-snapshot barevná tolerance 3, stejná jako již
existující tolerance jiného menu. Aktualizovány jsou pouze Apple referenční
obrázky, nikoli GTK kresba.

## Primární zdroje

- [SwiftUI defaultAction](https://developer.apple.com/documentation/swiftui/keyboardshortcut/defaultaction): Return a vizuální důraz výchozího tlačítka.
- [SwiftUI keyboardShortcut](https://developer.apple.com/documentation/swiftui/view/keyboardshortcut%28_%3A%29): rozsah zkratky a první nalezený cíl.
- [CoreText UI font](https://developer.apple.com/documentation/coretext/ctfontcreateuifontforlanguage%28_%3A_%3A_%3A%29): výběr systémového UI fontu podle účelu a jazyka.
- [CoreText font URL](https://developer.apple.com/documentation/coretext/kctfonturlattribute): veřejný způsob získání URL fontu.
- [Apple HIG Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility): rozměry cílů a alternativní způsoby ovládání.

Nejsilnější podklad pro zde uvedené konkrétní rozměry a události jsou přiložená
měření skutečného SwiftUI na výše uvedených verzích systému. Nejde o tvrzení
o všech verzích macOS a iOS.
