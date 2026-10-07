# Control remot

Fauste Player es pot llegir i manejar per la xarxa a través d'una API HTTP,
amb actualitzacions en directe, i a través d'OSC. Una pàgina web, una
aplicació de mòbil, l'automatització d'una emissora o una superfície de
control les poden usar. Està **desactivat** fins que l'actives, i al principi
només respon en aquest ordinador.

## Activar-lo {#turning-it-on}

![Configuració, Remot: l'API HTTP activada i escoltant en aquest ordinador, i l'OSC desactivat](../../images/guide/settings-remote.png)

Obre **Configuració → Remot** i marca **Permetre el control remot per HTTP**
(o **Permetre el control per OSC**). La línia sota cada interruptor diu si el
servidor està escoltant, i on, o per què no s'ha iniciat. Els canvis
s'apliquen a l'instant; no cal reiniciar. Un camp de text (una adreça, el
testimoni, una llista) s'aplica quan el deixes, obres una altra secció o
tanques Configuració; un valor que encara no és vàlid conserva el que s'està
usant, i Esc cancel·la el que has escrit.

També pots editar `config.json` mentre Fauste Player és tancat (vegeu
[Dades i còpies de seguretat](data-and-backups.md) per saber on és). Dins de
l'objecte `"config"`, posa `remote.http.enabled` a `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Escolta a `http://127.0.0.1:7380`. Iniciar-lo mai no reprodueix res; només
actuen les peticions.

## Escoltar a la xarxa de l'estudi {#listening-on-the-studio-network}

Per accedir-hi des d'altres ordinadors, posa `bind` a `0.0.0.0` (o a una de
les adreces d'aquest ordinador) i defineix un **testimoni** d'almenys 16
caràcters. A Configuració → Remot, **Generar** crea un testimoni aleatori
llarg. Està amagat fins que prems **Mostrar**, i **Copiar** el posa al
porta-retalls per al client. Sense testimoni el servidor es nega a iniciar-se,
i el registre en diu el motiu.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Els clients envien el testimoni com a `Authorization: Bearer <token>`. L'API
no està xifrada. Mantén-la en una xarxa d'estudi de confiança, o posa-la
darrere d'un servidor intermediari invers amb HTTPS.

## Pàgines web {#web-pages}

Una pàgina web servida des d'una altra adreça només pot usar l'API si el seu
origen (per exemple `https://studio.example`) és llistat a `cors_origins`.
Les peticions d'altres pàgines es refusen, fins i tot en aquest ordinador,
de manera que una pàgina que tinguis oberta per casualitat no pot manejar el
reproductor. `"*"` (qualsevol origen) només s'accepta juntament amb un
testimoni.

## Què pot fer un client {#what-a-client-can-do}

Un client pot:

- llegir els reproductors, les llistes, les pistes (amb caràtula i forma
  d'ona) i la cartutxera;
- reproduir, posar en pausa, aturar, fer fos, reiniciar i tornar enrere;
- triar l'entrada següent (també l'entrada en antena: es reprodueix una
  vegada més), preescoltar i cercar (en un reproductor aturat, una cerca tria
  on Play inicia l'entrada següent, i una cerca abans del seu cue-in comença
  al cue-in);
- definir volums, modes, stop al final de l'actual, i les marques de
  repetició i de stop al final d'una entrada;
- disparar, aturar i preescoltar cartutxos, i canviar la pàgina de cartutxos
  mostrada;
- editar: crear, canviar el nom i eliminar llistes; afegir una pista que ja és
  carregada, i treure, moure o duplicar entrades; crear, canviar el nom,
  canviar la mida i eliminar pàgines de cartutxos, i configurar un cartutx
  amb una pista carregada; definir o restablir marcadors.

Treure el que és en antena es refusa, com a la pantalla. Els fitxers que
encara no estan carregats no es poden afegir de manera remota: són en aquest
ordinador, de manera que afegeix-los primer aquí.

Un botó atenuat a la pantalla també es refusa de manera remota. La
referència completa és a [la documentació tècnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Prova-ho des d'un terminal:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Actualitzacions en directe {#live-updates}

Un client pot seguir els canvis a mesura que passen en lloc de preguntar una
vegada i una altra. `GET /api/v1/events` és un flux d'esdeveniments: primer
tot l'estat, després cada canvi de reproductor, llista, pista o cartutxera, i
els temps del que sona diverses vegades per segon.

    curl -sN http://127.0.0.1:7380/api/v1/events

Una pàgina web usa `EventSource`. Els navegadors no hi poden enviar el
testimoni com a capçalera, de manera que va a l'adreça:
`/api/v1/events?token=<token>`.

## OSC {#osc}

OSC és el protocol habitual de les superfícies de control, les taules
d'il·luminació i el programari de control d'espectacles. Activa'l amb
`remote.osc.enabled`. Escolta al port UDP 7381 d'aquest ordinador. Per
acceptar paquets d'altres ordinadors, posa `remote.osc.bind` a `0.0.0.0` i
llista-ne les adreces o subxarxes a `remote.osc.allowed_sources` (per exemple
`"192.168.1.0/24"`). OSC no té contrasenya, de manera que mantén-lo en una
xarxa d'estudi de confiança.

Els reproductors es numeren 1, 2, 3… tal com apareixen a la pantalla. Els
cartutxos es numeren a la pàgina mostrada.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Una superfície que vol mostrar l'estat (llums, noms, comptes enrere) se
subscriu, i aleshores rep cada valor una vegada i després només el que
canvia. Quan un reproductor o un botó de cartutx desapareix (menys
reproductors, una pàgina més petita), les seves adreces reben un valor buit
una vegada, de manera que la superfície les esborra. S'ha de tornar a
subscriure en un minut (`subscription_ttl_secs`) per continuar rebent:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Un subscriptor pot indicar qualsevol port de la seva pròpia adreça, i se'n
conserven fins a `max_subscribers`. Per tant, qualsevol que pugui enviar
també es pot subscriure. És un motiu més per mantenir OSC en una xarxa de
confiança.

`oscsend` i `oscdump` vénen amb liblo (`liblo-tools` a Debian i Ubuntu). La
llista completa d'adreces és a
[la documentació tècnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

## Tots els ajustos {#all-settings}

| Ajust | Per defecte | Significat |
|---|---|---|
| `remote.http.enabled` | `false` | Activa l'API |
| `remote.http.bind` | `127.0.0.1` | Adreça on escoltar (una adreça IP) |
| `remote.http.port` | `7380` | Port (1024–65535) |
| `remote.http.token` | buit | Necessari fora d'aquest ordinador; almenys 16 caràcters |
| `remote.http.cors_origins` | cap | Orígens web que poden cridar l'API |
| `remote.http.request_timeout_ms` | `10000` | El màxim que pot trigar una petició |
| `remote.http.max_body_bytes` | `65536` | El cos de petició més gran |
| `remote.http.max_event_clients` | `16` | Fluxos d'esdeveniments en directe alhora |
| `remote.osc.enabled` | `false` | Activa l'OSC |
| `remote.osc.bind` | `127.0.0.1` | Adreça on escoltar |
| `remote.osc.port` | `7381` | Port UDP (1024–65535) |
| `remote.osc.allowed_sources` | aquest ordinador | Adreces o subxarxes els paquets de les quals s'accepten |
| `remote.osc.max_subscribers` | `16` | Subscriptors alhora |
| `remote.osc.subscription_ttl_secs` | `60` | Una subscripció no renovada en aquest temps s'acaba |
| `remote.events.position_interval_ms` | `250` | Amb quina freqüència es publiquen els temps mentre sona |

Els valors fora d'interval es corregeixen quan es carrega el fitxer, i la
correcció es registra.
