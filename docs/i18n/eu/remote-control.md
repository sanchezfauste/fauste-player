# Urruneko kontrola

Fauste Player sarearen bidez irakurri eta erabil daiteke, HTTP API baten
bidez, zuzeneko eguneratzeekin, eta OSC bidez. Web-orri batek,
telefono-aplikazio batek, irrati-kate baten automatizazioak edo kontrol-gainazal
batek erabil ditzakete. **Desaktibatuta** dago zuk aktibatu arte, eta
hasieran ordenagailu honetan bakarrik erantzuten du.

## Aktibatzea {#turning-it-on}

![Ezarpenak, Urrunekoa: HTTP APIa aktibatuta eta ordenagailu honetan entzuten, eta OSC desaktibatuta](../../images/guide/settings-remote.png)

Ireki **Ezarpenak → Urrunekoa** eta markatu **Baimendu urruneko kontrola
HTTP bidez** (edo **Baimendu OSC bidezko kontrola**). Etengailu bakoitzaren
azpiko lerroak adierazten du zerbitzaria entzuten ari den ala ez, eta non,
edo zergatik ez den abiarazi. Aldaketak berehala aplikatzen dira; ez da
berrabiarazi behar. Testu-eremu bat (helbide bat, tokena, zerrenda bat)
uztean, beste atal bat irekitzean edo Ezarpenak ixtean aplikatzen da;
oraindik baliozkoa ez den balio batek erabiltzen ari dena mantentzen du, eta
Esc teklak idatzi duzuna bertan behera uzten du.

`config.json` ere edita dezakezu Fauste Player itxita dagoela (ikus
[Datuak eta babeskopiak](data-and-backups.md) non dagoen jakiteko).
`"config"` objektuaren barruan, ezarri `remote.http.enabled` `true` balioan:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

`http://127.0.0.1:7380` helbidean entzuten du. Abiarazteak ez du inoiz ezer
jotzen; eskaerek bakarrik eragiten dute.

## Estudioko sarean entzutea {#listening-on-the-studio-network}

Beste ordenagailu batzuetatik iristeko, ezarri `bind` `0.0.0.0` balioan
(edo ordenagailu honen helbideetako batean) eta ezarri gutxienez 16
karaktereko **token** bat. Ezarpenak → Urrunekoa atalean, **Sortu** botoiak
ausazko token luze bat sortzen du. Ezkutatuta dago **Erakutsi** sakatu
arte, eta **Kopiatu** botoiak arbelean jartzen du bezeroarentzat.
Tokenik gabe, zerbitzariak uko egiten dio abiarazteari, eta erregistroak
zergatik adierazten du.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Bezeroek tokena `Authorization: Bearer <token>` gisa bidaltzen dute. APIa
ez dago zifratuta. Mantendu estudioko sare fidagarri batean, edo jarri
HTTPS duen alderantzizko proxy baten atzean.

## Web-orriak {#web-pages}

Beste helbide batetik zerbitzatutako web-orri batek APIa erabil dezake
bere jatorria (adibidez `https://studio.example`) `cors_origins` zerrendan
badago bakarrik. Beste orrietako eskaerak baztertu egiten dira, ordenagailu
honetan ere bai; beraz, kasualitatez irekita duzun orri batek ezin du
erreproduzitzailea gidatu. `"*"` (edozein jatorri) token batekin batera
bakarrik onartzen da.

## Bezero batek zer egin dezakeen {#what-a-client-can-do}

Bezero batek hau egin dezake:

- erreproduzitzaileak, zerrendak, pistak (azalarekin eta uhin-formarekin)
  eta kartutxo-panela irakurri;
- jo, pausatu, gelditu, itzali, berrabiarazi eta atzera egin;
- hurrengo sarrera aukeratu (airean dagoen sarrera ere bai: beste behin
  jotzen da), aurrez entzun, eta jauzi egin (geldituta dagoen
  erreproduzitzaile batean, jauzi batek Play-k hurrengo sarrera non hasiko
  duen aukeratzen du, eta bere cue-in aurreko jauzi bat cue-in puntuan
  hasten da);
- bolumenak, moduak, uneko pistaren ondoren gelditzea, eta sarrera baten
  errepikapen- eta gelditze-markak ezarri;
- kartutxoak jaurti, gelditu eta aurrez entzun, eta erakutsitako
  kartutxo-orria aldatu;
- editatu: zerrendak sortu, izenez aldatu eta ezabatu; jada kargatuta
  dagoen pista bat gehitu, eta sarrerak kendu, mugitu edo bikoiztu;
  kartutxo-orriak sortu, izenez aldatu, tamainaz aldatu eta ezabatu, eta
  kartutxo bat kargatutako pista batekin konfiguratu; markatzaileak ezarri
  edo berrezarri.

Airean dagoena kentzea baztertu egiten da, pantailan bezala. Oraindik
kargatu ez diren fitxategiak ezin dira urrunetik gehitu: ordenagailu honetan
daude, beraz gehitu hemen lehenik.

Pantailan grisez dagoen botoi bat urrunetik ere baztertu egiten da.
Erreferentzia osoa [dokumentazio teknikoan](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md)
dago.

Probatu terminal batetik:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Zuzeneko eguneratzeak {#live-updates}

Bezero batek aldaketak gertatu ahala jarraitu ditzake, behin eta berriro
galdetu beharrean. `GET /api/v1/events` gertaera-fluxu bat da: lehenik egoera
osoa, ondoren erreproduzitzaile, zerrenda, pista edo kartutxo-panelaren
aldaketa bakoitza, eta jotzen ari denaren denborak segundoko hainbat
aldiz.

    curl -sN http://127.0.0.1:7380/api/v1/events

Web-orri batek `EventSource` erabiltzen du. Nabigatzaileek ezin dute
tokena goiburu gisa bidali hor, beraz helbidean doa:
`/api/v1/events?token=<token>`.

## OSC {#osc}

OSC kontrol-gainazalen, argiztapen-mahaien eta ikuskizun-kontroleko
softwarearen ohiko protokoloa da. Aktibatu `remote.osc.enabled` bidez.
Ordenagailu honetako 7381 UDP atakan entzuten du. Beste ordenagailu
batzuetako paketeak onartzeko, ezarri `remote.osc.bind` `0.0.0.0` balioan
eta zerrendatu haien helbideak edo azpisareak `remote.osc.allowed_sources`
aukeran (adibidez `"192.168.1.0/24"`). OSCk ez du pasahitzik; beraz,
mantendu estudioko sare fidagarri batean.

Erreproduzitzaileak 1, 2, 3… zenbakitzen dira, pantailan agertzen diren
bezala. Kartutxoak erakutsitako orrian zenbakitzen dira.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Egoera erakutsi nahi duen gainazal batek (argiak, izenak, atzerako
kontaketak) harpidetza egiten du, eta ondoren balio bakoitza behin jasotzen
du, eta gero aldatzen dena bakarrik. Erreproduzitzaile bat edo
kartutxo-botoi bat desagertzen denean (erreproduzitzaile gutxiago, orri txikiagoa),
haren helbideek balio huts bat jasotzen dute behin, gainazalak garbi
ditzan. Minutu baten barruan berriro harpidetu behar du
(`subscription_ttl_secs`) jasotzen jarraitzeko:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Harpidedun batek bere helbideko edozein ataka izenda dezake, eta gehienez
`max_subscribers` gordetzen dira. Beraz, bidaltzeko baimena duen edonork
harpidetu ere egin daiteke. Hori beste arrazoi bat da OSC sare fidagarri
batean mantentzeko.

`oscsend` eta `oscdump` liblo-rekin datoz (`liblo-tools` Debian eta
Ubuntun). Helbideen zerrenda osoa
[dokumentazio teknikoan](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc)
dago.

## Ezarpen guztiak {#all-settings}

| Ezarpena | Lehenetsia | Esanahia |
|---|---|---|
| `remote.http.enabled` | `false` | APIa aktibatzen du |
| `remote.http.bind` | `127.0.0.1` | Entzuteko helbidea (IP helbide bat) |
| `remote.http.port` | `7380` | Ataka (1024–65535) |
| `remote.http.token` | hutsik | Ordenagailu honetatik kanpo derrigorrezkoa; gutxienez 16 karaktere |
| `remote.http.cors_origins` | bat ere ez | APIari dei egin diezaioketen web-jatorriak |
| `remote.http.request_timeout_ms` | `10000` | Eskaera batek har dezakeen denborarik luzeena |
| `remote.http.max_body_bytes` | `65536` | Eskaera-gorputzik handiena |
| `remote.http.max_event_clients` | `16` | Aldi bereko zuzeneko gertaera-fluxuak |
| `remote.osc.enabled` | `false` | OSC aktibatzen du |
| `remote.osc.bind` | `127.0.0.1` | Entzuteko helbidea |
| `remote.osc.port` | `7381` | UDP ataka (1024–65535) |
| `remote.osc.allowed_sources` | ordenagailu hau | Paketeak onartzen zaizkien helbideak edo azpisareak |
| `remote.osc.max_subscribers` | `16` | Aldi bereko harpidedunak |
| `remote.osc.subscription_ttl_secs` | `60` | Denbora horretan berritzen ez den harpidetza amaitu egiten da |
| `remote.events.position_interval_ms` | `250` | Jotzen ari den bitartean denborak zenbatero argitaratzen diren |

Tartetik kanpoko balioak fitxategia kargatzean zuzentzen dira, eta
zuzenketa erregistroan idazten da.
