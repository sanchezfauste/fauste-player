# Datuak eta babeskopiak

## Fitxategiak non gordetzen diren {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Ezarpenak (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Zerrendak eta saioa | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Analisiaren cachea | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Erregistroak eta hutsegite-txostenak | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Linuxeko bide-izenek XDG aldagaiak (`XDG_CONFIG_HOME` eta abar) jarraitzen
dituzte ezarrita daudenean.

## Modu eramangarria {#portable-mode}

Ezarri `FAUSTE_HOME` ingurune-aldagaia karpeta batean, eta dena bertan
gordetzen da, `config/`, `data/`, `cache/` eta `logs/` karpetetan. Hori
erabilgarria da USB memoria batean, edo konfigurazio bereiziak bata
bestearen ondoan mantentzeko.

## Fitxategiak {#files}

| Fitxategia | Edukia |
|---|---|
| `config.json` | Ezarpenak, irteerak, analisiaren atalaseak, doikuntza aurreratua |
| `playlists.json` | Zerrendak, pistak, jotako markak, eskuzko markatzaileak |
| `session.json` | Erreproduzitzaile bakoitzeko: erakutsitako zerrenda, uneko eta hurrengo pistak, modua, posizioa, bolumena, zutabeen zabalerak (zutabeka) |

## Gordetze automatikoa eta babeskopiak {#autosave-and-backups}

- Aldaketak gertatu eta segundo bat ingurura gordetzen dira. Zerbait
  jotzen ari den bitartean, saioa (posizioak) erritmo berean eguneratzen
  da.
- Gordetze bakoitzak aldi baterako fitxategi bat idazten du eta ondoren
  zaharra ordezten du; beraz, argindar-etenaldi batek ez du inoiz erdi
  idatzitako fitxategirik uzten.
- Aurreko bertsioak `.bak1`, `.bak2` eta `.bak3` gisa gordetzen dira.
- Fitxategi bat ezin bada irakurri, babeskopia on berriena erabiltzen da.
  Fitxategi irakurtezina gorde egiten da, `*.corrupt-<time>` izenarekin,
  aztertu ahal izateko. Aplikazioa beti abiarazten da.

## Hutsegite baten ondoko berreskuratzea {#crash-recovery}

Hutsegite edo berrabiarazte baten ondoren, erreproduzitzaile bakoitza bere
zerrendarekin, uneko eta hurrengo pistekin eta posizioarekin itzultzen da,
baina **pausan edo geldituta**. Ez da ezer airera ateratzen bere kabuz.
Bere pistaren amaieran-amaieran zegoen erreproduzitzaile bat pista horren
hasieran itzultzen da, Play sakatzean pista jo dezan berehala amaitu
ordez.

## `config.json` eskuz editatzea {#editing-configjson-by-hand}

Itxi aplikazioa lehenik (audioa airean badago, galdetu egiten du itxi
aurretik). Balio ezezagunak edo tartetik kanpokoak baliozko balio
hurbilenera zuzentzen dira fitxategia kargatzean, eta zuzenketak
erregistroan idazten dira. `limits` eta `tuning` atalek balio aurreratuak
dituzte (baliabide-mugak, motorraren denborak), Ezarpenak leihoan ez
daudenak.
