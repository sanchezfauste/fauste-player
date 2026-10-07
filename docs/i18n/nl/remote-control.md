# Bediening op afstand

Fauste Player kan via het netwerk worden uitgelezen en bediend door middel van
een HTTP-API, met live-updates, en via OSC. Een webpagina, een telefoon-app,
de automatisering van een zender of een bedieningspaneel kan ze gebruiken. Ze
staat **uit** totdat je haar aanzet, en in eerste instantie antwoordt ze
alleen op deze computer.

## Aanzetten {#turning-it-on}

![Instellingen, Op afstand: de HTTP-API aan en luisterend op deze computer, en OSC uit](../../images/guide/settings-remote.png)

Open **Instellingen → Op afstand** en vink **Bediening op afstand via HTTP
toestaan** aan (of **Bediening via OSC toestaan**). De regel onder elke
schakelaar zegt of de server luistert, en waar, of waarom hij niet is
gestart. Wijzigingen werken meteen; herstarten is niet nodig. Een tekstveld
(een adres, het token, een lijst) wordt toegepast wanneer je het verlaat,
een andere sectie opent of Instellingen sluit; een waarde die nog niet
geldig is, laat de waarde in gebruik staan, en Esc annuleert wat je typte.

Je kunt ook `config.json` bewerken terwijl Fauste Player gesloten is (zie
[Gegevens en back-ups](data-and-backups.md) voor de locatie). Zet binnen het
object `"config"` de waarde `remote.http.enabled` op `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Ze luistert op `http://127.0.0.1:7380`. Het starten ervan speelt nooit iets
af; alleen verzoeken doen iets.

## Luisteren op het studionetwerk {#listening-on-the-studio-network}

Om het vanaf andere computers te bereiken, zet je `bind` op `0.0.0.0` (of op
een van de adressen van deze computer) en stel je een **token** van minstens
16 tekens in. In Instellingen → Op afstand maakt **Genereren** een lang
willekeurig token. Het blijft verborgen totdat je op **Tonen** drukt, en
**Kopiëren** zet het op het klembord voor de client.
Zonder token weigert de server te starten, en het logbestand zegt waarom.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Clients sturen het token mee als `Authorization: Bearer <token>`. De API is
niet versleuteld. Houd haar op een vertrouwd studionetwerk, of zet haar achter
een reverse proxy met HTTPS.

## Webpagina’s {#web-pages}

Een webpagina die vanaf een ander adres wordt aangeboden, kan de API alleen
gebruiken als zijn origin (bijvoorbeeld `https://studio.example`) in
`cors_origins` staat. Verzoeken van andere pagina’s worden geweigerd, zelfs op
deze computer, zodat een pagina die je toevallig open hebt staan de speler niet
kan aansturen. `"*"` (elke origin) wordt alleen samen met een token
geaccepteerd.

## Wat een client kan doen {#what-a-client-can-do}

Een client kan:

- de spelers, playlists, nummers (met hoes en golfvorm) en de cartwall lezen;
- afspelen, pauzeren, stoppen, faden, opnieuw starten en teruggaan;
- het volgende item kiezen (ook het item dat on air is: het speelt nog een
  keer), voorbeluisteren en zoeken (op een gestopte speler kiest een zoekactie
  waar Play het volgende item start, en een zoekactie vóór de cue-in start bij
  de cue-in);
- volumes, modi, stoppen na het huidige nummer en de markeringen voor herhalen
  en stoppen na een item instellen;
- carts starten, stoppen en voorbeluisteren, en de getoonde cartpagina
  wijzigen;
- bewerken: playlists aanmaken, hernoemen en verwijderen; een al geladen
  nummer toevoegen, en items verwijderen, verplaatsen of dupliceren; cartpagina’s
  aanmaken, hernoemen, het formaat wijzigen en verwijderen, en een cart
  instellen met een geladen nummer; markers instellen of terugzetten.

Verwijderen wat on air is, wordt geweigerd, net als op het scherm. Bestanden
die nog niet geladen zijn, kunnen niet op afstand worden toegevoegd: ze staan
op deze computer, dus voeg ze eerst hier toe.

Een knop die op het scherm grijs is, wordt ook op afstand geweigerd. De
volledige referentie staat in
[de technische documentatie](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Probeer het vanuit een terminal:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Live-updates {#live-updates}

Een client kan wijzigingen volgen terwijl ze gebeuren, in plaats van steeds
opnieuw te vragen. `GET /api/v1/events` is een stroom van gebeurtenissen:
eerst de hele toestand, daarna elke wijziging aan een speler, playlist,
nummer of de cartwall, en een paar keer per seconde de tijden van wat er
speelt.

    curl -sN http://127.0.0.1:7380/api/v1/events

Een webpagina gebruikt `EventSource`. Browsers kunnen het token daar niet als
header meesturen, dus het gaat in het adres:
`/api/v1/events?token=<token>`.

## OSC {#osc}

OSC is het gebruikelijke protocol van bedieningspanelen, lichtmengtafels en
showcontrolesoftware. Zet het aan met `remote.osc.enabled`. Het luistert op
UDP-poort 7381 van deze computer. Om pakketten van andere computers te
accepteren, zet je `remote.osc.bind` op `0.0.0.0` en vermeld je hun adressen
of subnetten in `remote.osc.allowed_sources` (bijvoorbeeld
`"192.168.1.0/24"`). OSC heeft geen wachtwoord, dus houd het op een vertrouwd
studionetwerk.

Spelers zijn genummerd 1, 2, 3… zoals ze op het scherm staan. Carts zijn
genummerd binnen de getoonde pagina.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Een bedieningspaneel dat de toestand wil tonen (lampjes, namen, aftellingen)
abonneert zich, en ontvangt dan elke waarde één keer en daarna alleen wat
verandert. Verdwijnt een speler of een cartknop (minder spelers, een kleinere
pagina), dan ontvangen zijn adressen één keer een lege waarde, zodat het
paneel ze wist. Het moet zich binnen een minuut opnieuw abonneren
(`subscription_ttl_secs`) om te blijven ontvangen:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Een abonnee kan elke poort van zijn eigen adres opgeven, en er worden er
maximaal `max_subscribers` bijgehouden. Iedereen die mag versturen, kan zich
dus ook abonneren. Dat is nog een reden om OSC op een vertrouwd netwerk te
houden.

`oscsend` en `oscdump` komen met liblo (`liblo-tools` op Debian en Ubuntu). De
volledige lijst met adressen staat in
[de technische documentatie](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

## Alle instellingen {#all-settings}

| Instelling | Standaard | Betekenis |
|---|---|---|
| `remote.http.enabled` | `false` | De API aanzetten |
| `remote.http.bind` | `127.0.0.1` | Adres om op te luisteren (een IP-adres) |
| `remote.http.port` | `7380` | Poort (1024–65535) |
| `remote.http.token` | leeg | Vereist buiten deze computer; minstens 16 tekens |
| `remote.http.cors_origins` | geen | Weborigins die de API mogen aanroepen |
| `remote.http.request_timeout_ms` | `10000` | Langste duur van een verzoek |
| `remote.http.max_body_bytes` | `65536` | Grootste verzoekbody |
| `remote.http.max_event_clients` | `16` | Live-eventstromen tegelijk |
| `remote.osc.enabled` | `false` | OSC aanzetten |
| `remote.osc.bind` | `127.0.0.1` | Adres om op te luisteren |
| `remote.osc.port` | `7381` | UDP-poort (1024–65535) |
| `remote.osc.allowed_sources` | deze computer | Adressen of subnetten waarvan pakketten worden geaccepteerd |
| `remote.osc.max_subscribers` | `16` | Abonnees tegelijk |
| `remote.osc.subscription_ttl_secs` | `60` | Een abonnement dat niet binnen deze tijd wordt vernieuwd, eindigt |
| `remote.events.position_interval_ms` | `250` | Hoe vaak tijden worden gepubliceerd tijdens het afspelen |

Waarden buiten het bereik worden bij het laden van het bestand gecorrigeerd,
en de correctie wordt gelogd.
