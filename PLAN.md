# uitdraai: plan

## Doel
Een lichte Markdown-previewer voor Linux, Wayland-first (niri, Hyprland), geschreven in Rust. Hij toont één bestand live met eigen CSS en exporteert naar PDF (DOCX/ODT volgt in fase 3). De CLI is de basis. De GUI is een dun GTK4-venster dat dezelfde kern gebruikt.

Werkafspraken (eenvoud, security, performance, Rust-praktijken) staan in `CLAUDE.md`.

## Buiten scope (v1)
Geen editor, geen tabs of meerdere bestanden, geen plugins, geen scroll-sync met een editor en geen X11-specifieke workarounds. Mermaid en KaTeX komen pas in fase 3.

## Architectuur
Eén crate met een `gui`-feature, zodat de CLI ook bouwt zonder WebKit-dependencies.

```
uitdraai/
├── Cargo.toml          # features: default = ["gui"], gui = ["gtk4", "webkit6", "async-channel"]
├── CLAUDE.md
├── PLAN.md
├── themes/             # standaardthema's, ingebakken via include_str!
│   ├── default.css
│   └── print.css       # @page, marges, paginanummers
└── src/
    ├── main.rs         # clap: subcommando's en dispatch
    ├── render.rs       # markdown → HTML-fragment en complete pagina
    ├── theme.rs        # thema's zoeken: --css, config-dir, ingebakken
    ├── export.rs       # PDF (weasyprint); DOCX/ODT via pandoc in fase 3
    ├── watch.rs        # file watching met debounce
    ├── config.rs       # ~/.config/uitdraai/config.toml (fase 3)
    └── gui/            # alleen met feature "gui"
        ├── mod.rs
        └── window.rs
```

### Crates
- `comrak`: Markdown met GFM (tabellen, taaklijsten, footnotes, strikethrough).
- `syntect`: code highlighting, gekoppeld aan comrak via de syntect-adapter, met CSS-classes.
- `clap` (derive): CLI.
- `notify` + `notify-debouncer-full`: file watching.
- `serde`, `serde_json`, `toml`: configuratie en veilig escapen richting JavaScript.
- `directories`: config-paden volgens XDG.
- `anyhow`: foutafhandeling.
- `gtk4` + `webkit6`: GUI (alleen met feature `gui`).
- `async-channel`: resultaten van de render-thread naar de GTK main thread (alleen met feature `gui`; vervangt het verwijderde `glib::MainContext::channel`).

Nieuwe crates alleen na overleg (zie `CLAUDE.md`).

### Ontwerpkeuzes
**Eén renderpad.** `render.rs` levert een complete HTML-pagina met CSS inline. Preview en PDF-export krijgen exact dezelfde HTML en CSS. Let op: ze worden wel door twee verschillende engines getekend (WebKitGTK voor de preview, WeasyPrint voor de PDF), dus kleine verschillen zijn mogelijk. Houd thema's daarom bij documenttypografie en vermijd complexe layout (grid, geavanceerde flexbox).

**Geen JavaScript in de pagina.** De preview heeft geen JS nodig. Alles wat dynamisch lijkt (highlighting, straks diagrammen en formules) wordt vooraf in Rust gerenderd. Alleen de live reload gebruikt `evaluate_javascript` vanuit de app zelf.

**Viewer los van de rest.** Alle WebKit-code blijft binnen `src/gui/`. `render.rs`, `export.rs` en `watch.rs` weten niets van WebKit. Zo kan de viewer later worden vervangen (zie Blitz in fase 3) zonder de rest te raken. Er komt nu géén trait of abstractielaag voor; dat pas als er echt een tweede viewer bijkomt.

**Rauwe HTML standaard uit.** Markdown wordt als onbetrouwbare input behandeld. Rauwe HTML in Markdown wordt niet doorgelaten, tenzij de gebruiker `--allow-html` meegeeft. Remote content wordt standaard geblokkeerd, onder meer via een CSP-meta in de pagina; aanzetten kan met `--allow-remote`.

**Relatieve afbeeldingen moeten werken.** In de GUI laad je de HTML met `load_html(html, Some("file:///pad/naar/map/"))`, en WeasyPrint krijgt `--base-url` mee.

**Externe tools voor export.** PDF gaat via het `weasyprint`-binary, vanwege de beste ondersteuning voor print-CSS (`@page`, `counter(page)`, page breaks). WeasyPrint wordt vooraf gedetecteerd; ontbreekt het, dan volgt een duidelijke foutmelding met installatietip.

**DOCX/ODT pas in fase 3.** Weinig gebruikt, en pandoc leest de Markdown direct, dus buiten `render.rs` om. Het is een bewuste uitzondering op "één renderpad": geen CSS, Word-opmaak via `--reference-doc`. Zonder `--allow-html` wordt het `pandoc -f gfm-raw_html`.

**Watch de map, niet het bestand.** Veel editors (Vim, Helix) slaan atomisch op via rename, waardoor een watch op het bestand zelf na de eerste save stilvalt. Watch daarom de parent directory, filter op bestandsnaam en debounce met ongeveer 100 ms.

**Live reload zonder flikkeren.** Bij een wijziging wordt niet de hele pagina herladen, alleen de inhoud van `#content` via `evaluate_javascript`. De HTML wordt daarbij geëscaped met `serde_json::to_string`. De scrollpositie blijft zo vanzelf behouden. Een volledige reload is alleen nodig bij een themawissel.

## CLI-specificatie
```
uitdraai <file.md>                         # opent GUI (met feature gui)
uitdraai render <file.md> [-o out.html]    # HTML naar bestand of stdout
uitdraai export <file.md> --pdf [-o <dir>]
uitdraai export <file.md> --pdf --watch    # herexporteer bij elke save
uitdraai themes                            # lijst beschikbare thema's

Globale opties:
  --css <pad>        eigen stylesheet (gaat voor --theme)
  --theme <naam>     thema uit config-dir of ingebakken
  --allow-html       rauwe HTML in Markdown toestaan
  --allow-remote     remote afbeeldingen laden (standaard geblokkeerd)
  --timing           duur per stap loggen naar stderr
```

Zoekvolgorde voor thema's: `--css`, dan `~/.config/uitdraai/themes/<naam>.css`, dan de ingebakken thema's. `print.css` wordt altijd vóór het thema geladen, zodat een `@media print`-blok in een eigen thema de print-regels kan overschrijven.

## GUI-specificatie
- App-id `io.github.<gebruikersnaam>.uitdraai`, zodat er window rules op gemaakt kunnen worden.
- Minimale toolbar met themadropdown en een knop voor PDF-export. Verbergbaar met `--no-toolbar` of `Ctrl+T`, want op een tiling WM wil je vaak alleen de content zien.
- Sneltoetsen: `Ctrl+O` openen, `Ctrl+E` PDF, `Ctrl+R` volledig herladen, `Ctrl+T` toolbar, `Ctrl+Q` sluiten, `Ctrl++` / `Ctrl+-` / `Ctrl+0` zoom via `WebView::set_zoom_level`.
- Na een export een korte melding in het venster, geen popup.
- Externe links openen in de standaardbrowser; alle andere navigatie wordt geblokkeerd (via de `decide-policy`-signal).

Voorbeelden van window rules:

```kdl
// niri
window-rule {
    match app-id="^io\\.github\\..*\\.uitdraai$"
    default-column-width { proportion 0.4; }
}
```

```
# Hyprland
windowrulev2 = float, class:^(io\.github\..*\.uitdraai)$
windowrulev2 = size 40% 90%, class:^(io\.github\..*\.uitdraai)$
```

## Fases

### Fase 1: kern + CLI
Renderen en exporteren werkt vanuit de terminal.

- [ ] Cargo-project met `gui`-feature (nog leeg) en release profile uit `CLAUDE.md`
- [ ] `render.rs`: comrak (GFM), syntect met CSS-classes, rauwe HTML standaard uit
- [ ] Standaardthema's `default.css` en `print.css`, inclusief highlighting-CSS
- [ ] `theme.rs` met de zoekvolgorde
- [ ] `export.rs`: PDF via weasyprint, met tool-detectie
- [ ] `watch.rs` met directory-watch en debounce, plus `export --watch`
- [ ] `--timing` vlag
- [ ] Tests voor rendering en themaresolutie; smoke-test voor export die wordt overgeslagen als de tools ontbreken

**Klaar als:** `uitdraai export notes.md --pdf` een correcte PDF oplevert, relatieve afbeeldingen in de PDF zichtbaar zijn, en een `<script>` in het Markdown-bestand niet in de output belandt.

### Fase 2: GUI
Een live previewvenster op Wayland.

- [ ] GTK4-applicatie met app-id, WebView en `load_html` met base URI
- [ ] WebView-instellingen dichtgezet volgens `CLAUDE.md`
- [ ] Watcher en rendering in een eigen thread, resultaat via `async-channel` naar de UI, ontvangen met `glib::spawn_future_local`
- [ ] Inhoud vervangen via JS voor reload zonder scroll-sprong
- [ ] Toolbar met themakiezer en exportknoppen, plus sneltoetsen
- [ ] Navigatiebeleid: externe links naar de browser, de rest blokkeren

**Klaar als:** je in niri of Hyprland een bestand opent, het in je editor opslaat en de preview binnen de performancedoelen bijwerkt zonder te verspringen.

### Fase 3: extra's
- [ ] Mermaid en KaTeX vooraf in Rust naar SVG renderen, zonder JavaScript in de pagina. Welke crate of tool daarvoor geschikt is, eerst uitzoeken en voorstellen
- [ ] Blitz heroverwegen als lichtere viewer (pure Rust, geen WebKit, geen webproces). Pas zinvol als de CSS die de thema's gebruiken daar goed wordt ondersteund; de roadmap van Blitz eerst naast de thema's leggen
- [ ] Automatisch licht/donker volgen via `prefers-color-scheme`
- [ ] DOCX/ODT-export via pandoc (`--docx`, `--odt`, `--reference-doc`), zie de ontwerpkeuze "DOCX/ODT pas in fase 3". Pandoc's `--sandbox` voor remote content uitzoeken
- [ ] `config.toml` voor standaardthema, exportmap, reference-doc en de NVIDIA-workaround

## Dependencies per distro
- **Arch:** `gtk4 webkitgtk-6.0 pandoc python-weasyprint`
- **Fedora:** `gtk4-devel webkitgtk6.0-devel pandoc weasyprint`
- **Debian/Ubuntu 24.04+:** `libgtk-4-dev libwebkitgtk-6.0-dev pandoc weasyprint`

## Risico's en open punten
- **NVIDIA:** WebKitGTK kan een leeg venster tonen. Workaround: `WEBKIT_DISABLE_DMABUF_RENDERER=1`, in fase 3 als optie in `config.toml`.
- **WeasyPrint vergt Python op het systeem.** Besloten: we accepteren dit, omdat WeasyPrint de beste ondersteuning voor print-CSS heeft en Rust geen volwassen eigen alternatief kent. WebKits `PrintOperation` en Typst zijn overwogen en afgewezen (beperkte print-CSS, respectievelijk geen CSS-styling).
- **Preview en PDF gebruiken verschillende engines.** Zie de ontwerpkeuze "Eén renderpad". Als verschillen in de praktijk storend worden, kan een optionele `--pdf-engine webkit` worden toegevoegd.
- **WebKitGTK is groot.** Meer dan 100 MB op schijf en ongeveer 80 tot 150 MB RAM per venster. Geaccepteerd voor nu; Blitz is het alternatief voor later (fase 3). Wie alleen de CLI wil, bouwt zonder de `gui`-feature.
- **Mermaid en KaTeX zonder JavaScript.** Vooraf renderen naar SVG in Rust is nog niet uitgezocht. Lukt dat niet goed, dan is de terugvaloptie eigen, ingebakken scripts via `UserContentManager` (nooit scripts uit het Markdown-bestand). Beslissen bij de start van fase 3.
- **Remote content.** Besloten: standaard uit, alleen aan via `--allow-remote`. In de preview blokkeert de CSP-meta het (zie `CLAUDE.md`). Voor WeasyPrint in fase 1 uitzoeken hoe je fetchen via de CLI uitzet; de CSP-meta geldt daar niet.
