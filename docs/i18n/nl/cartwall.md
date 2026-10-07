# Cartwall

De cartwall is de strook met knoppen onder de spelers. Elke knop, een
**cart**, speelt direct één geluid af: jingles, effecten, spots. Carts spelen
op hun eigen uitgangen, onafhankelijk van de spelers.

![De cartwall met één cart die speelt](../../images/guide/cartwall.png)

## Gebruik {#using-it}

- **Klik op een cart** om hem te starten. **Klik er nogmaals op** om hem te
  stoppen.
- **Alles stoppen** (rechts op de balk) stopt elke cart die speelt, op elke
  pagina. Het label toont hoeveel er spelen, zoals in **Alles stoppen (2)**;
  als er geen speelt, is de knop gedimd en staat er geen aantal.
- Terwijl een cart speelt, wordt zijn rand rood, krimpt een rode balk mee met
  het afspelen en telt zijn tijd af.
- Carts **overlappen** standaard: een tweede starten stopt de eerste niet. Een
  cart met **Andere carts stoppen bij starten** stopt eerst elke andere cart
  die on air is, op welke pagina ook.
- Een cart met **Herhalen (loop)** begint zonder onderbreking opnieuw bij zijn
  cue-in wanneer hij het einde bereikt, totdat je hem stopt.
- **Klik met de rechtermuisknop** op een cart voor meer opties:

| Item | Actie |
|---|---|
| Voorbeluisteren op CUE | Afspelen op de CUE-uitgang van de cartwall (gedimd als de cartwall naast de Main-uitgang geen Cue-uitgang heeft) |
| Stoppen | Stoppen |
| Bewerken… | In Instellingen openen |

Het menu van een lege cart heeft alleen **Bewerken…**, om zijn bestand te
kiezen.

- **Pagina’s:** de tabbladen naast **CARTWALL** wisselen van pagina. Een rode
  stip laat zien dat een cart op die pagina speelt.
- Klik op **CARTWALL** om de strook in te klappen of weer uit te klappen.
- Is het venster laag, dan worden de knoppen kleiner (tot een minimale
  hoogte) zodat alle ingestelde rijen passen; de cartwall scrolt pas als zelfs
  de kleinste knoppen niet passen.

| Uiterlijk van de knop | Betekenis |
|---|---|
| Violette stip | Jingle |
| Amberkleurige stip | Effect |
| Grijze stip | Spot (reclame) |
| ↻ na het type | Herhaalt |
| ✋ na het type | Stopt de andere carts bij starten |
| Bestand met een kruis / waarschuwingsteken | Het bestand ontbreekt / kan niet worden gedecodeerd; beweeg de muisaanwijzer over de cart voor de reden en het pad. Een ontbrekend bestand wordt elke 30 s opnieuw gezocht (`tuning.missing_recheck_ms`). |
| “Leeg”, gedimd | Geen bestand toegewezen |

Carts gebruiken dezelfde markers als nummers. Ze beginnen bij hun cue-in en
eindigen bij hun cue-out, die je kunt bewerken op de golfvorm van een speler
wanneer het bestand daar geladen is.

## Toetsenbord {#keyboard}

Standaard starten **F1**…**F12** cart 1–12 van de getoonde pagina, en stopt
**Ctrl+Space** alle carts (hetzelfde als **Alles stoppen**; het stopt ook een
cart die je op CUE voorbeluistert, zelfs als er geen cart speelt). Zie
[Toetsenbord](keyboard.md) om ze te wijzigen.

## Carts instellen {#setting-up-carts}

Ga naar **Instellingen → Cartwall**:

- **Pagina’s:** aanmaken, hernoemen, verwijderen (de laatste pagina kan niet
  worden verwijderd) en de rastergrootte instellen (rijen × kolommen). Een
  kleiner raster wordt geweigerd als het carts met een bestand zou laten
  vallen.
- **Importeren… / Exporteren…** slaan een pagina op in een
  `.cartpage.json`-bestand en laden die weer in, bijvoorbeeld om hem tussen
  studio’s te delen. Relatieve bestandspaden worden opgelost vanaf de map van
  het bestand.
- **Carts:** klik in het raster op een cart en stel dan zijn naam, bestand
  (**Kiezen…** of **Wissen**), type, **Herhalen (loop)** en **Andere carts
  stoppen bij starten** in.

De eigen Main- en Cue-uitgangen van de cartwall staan in **Instellingen →
Audio-uitgangen** (de rij **Cartwall**).
