# Werkafspraken voor uitdraai

Het plan en de fases staan in `PLAN.md`. Dit bestand beschrijft hoe we bouwen.

## Werkwijze
- Werk één fase uit `PLAN.md` tegelijk en stop daarna voor review.
- Vink afgeronde taken af in `PLAN.md`.
- `cargo fmt`, `cargo clippy --all-targets -- -D warnings` en `cargo test` moeten slagen voordat je een stap klaar meldt.
- Kleine, logische commits met een duidelijke boodschap per stap.
- Twijfel je over een ontwerpkeuze of een open punt uit `PLAN.md`: vraag het, gok niet.

## Eenvoud (gaat voor DRY)
- Geen trait of abstractie voor iets met maar één implementatie.
- Geen nieuwe crate zonder dat je het eerst voorstelt en uitlegt waarom.
- Duplicatie is oké tot het de derde keer voorkomt; dan pas samenvoegen.
- Liever een iets langere functie die leesbaar is dan drie kleine die je moet volgen.

## Eén bron van waarheid
- Er is precies één renderpad (`render.rs`). Preview en export gebruiken het allebei.
- Thema's en defaults staan op één plek; nergens hardcoded CSS in Rust-code.
- Alle WebKit-code blijft binnen `src/gui/`. `render.rs`, `export.rs` en `watch.rs` mogen niets van WebKit of GTK weten.

## Security
- Behandel elk `.md`-bestand als onbetrouwbare input.
- Geen JavaScript in de gerenderde pagina. Highlighting (en later diagrammen en formules) wordt vooraf in Rust gerenderd. De enige JS is de live reload die de app zelf via `evaluate_javascript` uitvoert.
- comrak: rauwe HTML standaard uit (`render.unsafe_ = false`). Alleen aan via de expliciete `--allow-html` vlag.
- WebView-instellingen:
  - `enable-javascript-markup = false` (`evaluate_javascript` blijft werken voor de live reload)
  - `allow-file-access-from-file-urls = false`
  - `allow-universal-access-from-file-urls = false`
- `decide-policy`: alleen de initiële load toestaan; externe links naar de systeembrowser, alle andere navigatie blokkeren.
- HTML die via `evaluate_javascript` wordt doorgegeven altijd escapen met `serde_json::to_string`, nooit via string-concatenatie.
- Subprocessen (pandoc, weasyprint) altijd via `std::process::Command` met losse args, nooit via een shell. Zet `--` vóór bestandsnamen, zodat een bestand dat met een streepje begint niet als optie wordt gelezen.
- Geen netwerkverkeer tijdens renderen, preview of exporteren. Remote content alleen via de expliciete `--allow-remote` vlag.
- De gerenderde pagina krijgt een CSP-meta: `default-src 'none'; img-src file: data:; style-src 'unsafe-inline'`. Met `--allow-remote` komt `https:` bij `img-src`. Dat blokkeert scripts en remote content ook als een andere laag faalt.
- Geen `unwrap()` of `expect()` buiten tests; fouten via `anyhow` met `.context(...)`.
- `cargo audit` draaien voordat er een dependency bijkomt.

## Performance
Doelen (release build, document van ~1000 regels):
- Venster zichtbaar met content: < 300 ms na start.
- Preview bijgewerkt na save: < 50 ms (exclusief debounce).
- Meet dit met de `--timing` vlag, die per stap de duur logt naar stderr.

Regels:
- Optimaliseer alleen wat gemeten traag is, met uitzondering van de punten hieronder.
- Syntect `SyntaxSet` en comrak-opties één keer opbouwen (`std::sync::LazyLock`), nooit per render.
- Highlighting met CSS-classes (`ClassedHTMLGenerator`), geen inline styles. Dat geeft kleinere HTML, en wisselen van thema kan zonder opnieuw te renderen.
- Renderen gebeurt buiten de GTK main thread; alleen het resultaat gaat via een channel naar de UI.
- Bij live reload alleen de inhoud van `#content` vervangen, nooit de hele pagina herladen.

Release profile in `Cargo.toml`:

```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

## Rust-praktijken
- Edition 2024. `Cargo.lock` wordt meegecommit (het is een binary).
- Publieke functies krijgen een korte doc-comment; geen comments die alleen herhalen wat de code al zegt.
- Unit tests naast de code (`mod tests`) voor logica; integratietests in `tests/` voor de CLI.
- Tests die externe tools (pandoc, weasyprint) nodig hebben, worden overgeslagen als die tools ontbreken, in plaats van te falen.
