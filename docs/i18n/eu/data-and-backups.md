# Datuak eta babeskopiak

## Fitxategiak non gordetzen diren {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Ezarpenak (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Liburutegia, zerrendak, kartutxoak eta saioa (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Analisiaren cachea | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Erregistroak eta hutsegite-txostenak | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Linuxeko bide-izenek XDG aldagaiak (`XDG_CONFIG_HOME` eta abar) jarraitzen
dituzte ezarrita daudenean.

Snap-ak bere fitxategiak bere karpetan gordetzen ditu: ezarpenak
`~/snap/fauste-player/current/.config/fausteplayer/` karpetan, datu-basea eta
erregistroak `~/snap/fauste-player/current/.local/share/fausteplayer/`
karpetan, eta analisiaren cachea
`~/snap/fauste-player/common/.cache/fausteplayer/` karpetan.
`sudo snap remove fauste-player` komandoak horiek ere ezabatzen ditu
(snapshot bat gordetzen du denbora batez; `--purge` aukerak ez).

## Modu eramangarria {#portable-mode}

Ezarri `FAUSTE_HOME` ingurune-aldagaia karpeta batean, eta dena bertan
gordetzen da, `config/`, `data/`, `cache/` eta `logs/` karpetetan. Hori
erabilgarria da USB memoria batean, edo konfigurazio bereiziak bata
bestearen ondoan mantentzeko.

## Fitxategiak {#files}

| Fitxategia | Edukia |
|---|---|
| `config.json` | Ezarpenak, irteerak, analisiaren atalaseak, doikuntza aurreratua |
| `fauste.db` | Liburutegia (pistak, haien kategoriak, eskuzko markatzaileak, etiketak, ozentasuna), zerrendak beren jotako markekin, kartutxo-orriak eta saioa: erreproduzitzaile bakoitzeko erakutsitako zerrenda, uneko eta hurrengo pistak, modua, posizioa, bolumena eta zutabeen zabalerak (zutabeka), eta leihoaren diseinua |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Datu-basearen kopiak (ikus behean) |
| `window.ron` | Leiho nagusiaren tamaina, posizioa eta maximizatuta zegoen ala ez (ordenagailuan bakarrik; Waylanden sistemak aukeratzen du posizioa) |

## Gordetze automatikoa eta babeskopiak {#autosave-and-backups}

- Aldaketak gertatu eta segundo bat ingurura gordetzen dira, eta aldatu dena
  bakarrik idazten da. Zerbait jotzen ari den bitartean, saioa (posizioak)
  erritmo berean eguneratzen da.
- Datu-basea transakzioetan idazten da; beraz, argindar-etenaldi batek ez du
  inoiz erdi gordetako aldaketarik uzten.
- Saio bakoitzeko lehen gordetzean, eta gero bost minuturo gutxi gorabehera
  aldaketak gordetzen diren bitartean, aplikazioak datu-basea
  `fauste.db.bak1` fitxategira kopiatzen du eta kopia zaharragoak gora
  mugitzen ditu, `.bak2` eta `.bak3` izenetara. Beraz, segidan hainbat aldiz
  berrabiarazteak babeskopia zaharrenak kanporatzen ditu.
- Datu-basea ezin bada irakurri, babeskopia on berriena erabiltzen da.
  Fitxategi irakurtezina gorde egiten da, `fauste.db.corrupt-<time>`
  izenarekin, aztertu ahal izateko. Kopia guztiek huts egiten badute,
  aplikazioa hutsik abiarazten da. Beti abiarazten da.
- Erabili ezin den kopia bat ere gorde egiten da: aplikazioa beste kopia
  batetik (edo datu-base huts batetik) abiarazten denean, erabilgarri ez
  daudenak `fauste.db.bak1.corrupt-<time>` (edo `.newer-<time>`) izenarekin
  berrizendatzen dira, hurrengo babeskopiak inoiz kanpora ez ditzan.
- Aplikazioaren beste kopia batek datu-basea erabiltzen ari bada, kopia honek
  gorde gabe funtzionatzen du eta hala dioenik abiaraztean.
- Aplikazioaren bertsio berriago batek idatzitako datu-basea ez da irekitzen:
  `fauste.db.newer-<time>` izenarekin gordetzen da, eta aplikazioa irakur
  dezakeen babeskopia berrienetik abiarazten da, edo hutsik babeskopiarik ez
  badago; horrela, datuak bertsio zaharrago batekin irekitzeak ez ditu inoiz
  suntsitzen.
- `config.json` lehen bezala gordetzen da: aldi baterako fitxategi batek
  zaharra ordezten du, eta aurreko bertsioak `.bak1`, `.bak2` eta `.bak3`
  gisa gordetzen dira haren ondoan.
- Datu-basea saio batean batere erabili ezin bada (adibidez, ezin delako
  berrizendatu karpeta irakurtzeko soilik delako), aplikazioak gorde gabe
  funtzionatzen du hurrengo abiarazteraino, eta erregistroak hala dio.

## Bertsio zahar bateko zerrendak eta saioa {#playlists-and-session-from-an-earlier-version}

Bertsio zaharragoek zerrendak `playlists.json` fitxategian, kartutxo-orriak
`carts.json` fitxategian eta saioa `session.json` fitxategian gordetzen
zituzten. Bertsio honen lehen abiarazteak horiek (beren `.bak` kopiekin) behin
irakurtzen ditu eta dena `fauste.db` fitxategian jartzen du: liburutegia bere
markatzaile, etiketa eta ozentasunarekin, zerrendak beren jotako markekin,
kartutxo-orriak eta saioa. Gero fitxategi zaharrak `.imported` bukaerarekin
berrizendatzen dira (`playlists.json.imported`,
`playlists.json.bak1.imported`, eta abar). Ez dira gehiago erabiltzen, eta
ezabatu ditzakezu dena hor dagoela ziur zaudenean.

Bertsio zaharrago batera itzultzeko:

1. Itxi aplikazioa.
2. Mugitu `fauste.db`, `fauste.db-wal`, `fauste.db-shm` eta `fauste.db.bak*`
   guztiak datu-karpetatik kanpoko karpeta batera. Ez ezabatu.
3. Kendu `.imported` fitxategi zaharren izenei.
4. Abiarazi bertsio zaharragoa.

Eguneratzetik hona gorde den guztia fitxategiak mugitu dituzun karpetan
bakarrik geratzen da: fitxategi zaharrek ez dute. Geroago berriro
eguneratzen baduzu, fitxategi zaharrak berriro inportatzen dira, datu-karpetan
`fauste.db` edo `fauste.db.bak*` fitxategirik ez dagoen bitartean.

Abiarazte bat inportazioan zehar eten bada, hurrengoak amaitzen du. Fitxategi
zaharretako bat irakurri ezin bada (handiegia delako, edo bertsio berriago
batek idatzi duelako), aplikazioak ez du ezer inportatzen eta saio hori gorde
gabe funtzionatzen du, beraz fitxategi zaharrak dauden tokian geratzen dira;
erregistroak fitxategiaren izena ematen du. Konpondu edo kendu eta abiarazi
berriro. `fauste.db` existitzen denetik, JSON fitxategi zaharrak ez dira
inoiz berriro inportatzen.

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
