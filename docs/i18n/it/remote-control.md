# Controllo remoto

Fauste Player si può leggere e usare in rete tramite un'API HTTP, con
aggiornamenti in tempo reale, e tramite OSC. Possono usarli una pagina web,
un'app per telefono, l'automazione di una stazione o una superficie di
controllo. È **disattivato** finché non lo attivi, e all'inizio risponde solo
su questo computer.

## Attivarlo {#turning-it-on}

![Impostazioni, Remoto: l'API HTTP attiva e in ascolto su questo computer, e OSC disattivato](../../images/guide/settings-remote.png)

Apri **Impostazioni → Remoto** e spunta **Consenti il controllo remoto via
HTTP** (o **Consenti il controllo via OSC**). La riga sotto ogni
interruttore dice se il server è in ascolto, e dove, o perché non si è
avviato. Le modifiche si applicano subito; non serve riavviare. Un campo di
testo (un indirizzo, il token, un elenco) si applica quando lo lasci, apri
un'altra sezione o chiudi le Impostazioni; un valore non ancora valido
mantiene quello in uso, ed Esc annulla ciò che hai digitato.

Puoi anche modificare `config.json` mentre Fauste Player è chiuso (vedi
[Dati e backup](data-and-backups.md) per sapere dove si trova). Dentro
l'oggetto `"config"`, imposta `remote.http.enabled` su `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

Ascolta su `http://127.0.0.1:7380`. Avviarlo non riproduce mai nulla; agiscono
solo le richieste.

## In ascolto sulla rete dello studio {#listening-on-the-studio-network}

Per raggiungerlo da altri computer, imposta `bind` su `0.0.0.0` (o su uno
degli indirizzi di questo computer) e imposta un **token** di almeno 16
caratteri. In Impostazioni → Remoto, **Genera** crea un token casuale
lungo. Resta nascosto finché non premi **Mostra**, e **Copia** lo mette
negli appunti per il client.
Senza un token il server rifiuta di avviarsi, e il log dice perché.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

I client inviano il token come `Authorization: Bearer <token>`. L'API non è
cifrata. Tienila su una rete di studio fidata, oppure mettila dietro un
reverse proxy con HTTPS.

## Pagine web {#web-pages}

Una pagina web servita da un altro indirizzo può usare l'API solo se la sua
origine (per esempio `https://studio.example`) è elencata in `cors_origins`.
Le richieste da altre pagine vengono rifiutate, anche su questo computer,
così una pagina che capita di avere aperta non può pilotare il lettore.
`"*"` (qualsiasi origine) è accettato solo insieme a un token.

## Cosa può fare un client {#what-a-client-can-do}

Un client può:

- leggere i lettori, le playlist, i brani (con copertina e forma d'onda) e
  la cartwall;
- riprodurre, mettere in pausa, fermare, dissolvere, riavviare e tornare
  indietro;
- scegliere la voce successiva (anche la voce in onda: suona ancora una
  volta), preascoltare (un preascolto che si sentirebbe in onda mentre lì suona qualcosa viene rifiutato, a meno che la richiesta non dica `"force": true`; vedi il riferimento dell'API) e spostarsi (su un lettore fermo uno spostamento
  sceglie da dove Play avvia la voce successiva, e uno spostamento prima del
  suo cue-in parte dal cue-in);
- impostare volumi, modalità, stop alla fine del brano attuale, e i segni di
  ripetizione e di stop dopo di una voce;
- lanciare, fermare e preascoltare i cart, e cambiare la pagina di cart
  mostrata;
- modificare: creare, rinominare ed eliminare playlist; aggiungere un brano
  già caricato, e rimuovere, spostare o duplicare voci; creare, rinominare,
  ridimensionare ed eliminare pagine di cart, e configurare un cart (nome,
  tipo, colore, loop, esclusivo) con un brano caricato; impostare o
  reimpostare i marker; escludere un brano dalla
  [normalizzazione del loudness](loudness.md) o regolarne il trim.

Rimuovere ciò che è in onda viene rifiutato, come sullo schermo. I file non
ancora caricati non si possono aggiungere da remoto: si trovano su questo
computer, quindi aggiungili prima qui.

Anche un pulsante disattivato sullo schermo viene rifiutato da remoto.

Un lettore la cui uscita Main è [bit-perfect](bit-perfect.md#what-it-costs)
continua a funzionare da remoto, con gli stessi limiti che sullo schermo:
`play` e `previous` cambiano brano subito, senza dissolvenza incrociata, e il
suo volume è mantenuto al 100 %. Un volume inferiore al 100 % viene rifiutato
(`409`, con il motivo), e esattamente il 100 % viene accettato. Ogni lettore
dice se il suo volume è mantenuto (`fader_locked`).

Un volume inviato come posizione del fader (il `fader` dell'API, `/volume` di
OSC) è una posizione sulla scala propria del fader sullo schermo, da 0
(silenzio) a 1 (0 dB): 0,9 è circa −2,5 dB, 0,8 circa −5,3 dB e 0,5 circa
−16 dB. Le versioni precedenti usavano un'altra scala (0,8 era −12 dB, 0,5
era −30 dB), quindi uno script scritto per loro ora imposta livelli più
alti: controlla i suoi valori.

Ogni brano porta la sua misura di loudness (`loudness`, `null` finché non è
misurato) e la sua impostazione (`loudness_edit`: `excluded` e `trim_db`). Un
lettore in riproduzione o in pausa, e ogni cart in riproduzione, dice con
quale guadagno di [normalizzazione](loudness.md) è partito il suo brano e
perché (`normalization`: `gain_db` e uno `status` come `applied`,
`not_measured` o `bit_perfect`). `PUT /api/v1/tracks/<id>/loudness` con
`{"excluded": false, "trim_db": -1.5}` cambia l'impostazione, come fa il
popup **Loudness…**; un trim oltre il trim massimo viene rifiutato (`409`,
con il motivo). Le impostazioni della normalizzazione stessa (attiva o no, il
valore di riferimento, i limiti) si cambiano solo su questo computer. OSC e
MIDI non hanno controlli di loudness.

Il riferimento completo è nella [documentazione tecnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Provalo da un terminale:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Aggiornamenti in tempo reale {#live-updates}

Un client può seguire i cambiamenti man mano che avvengono invece di
chiedere ancora e ancora. `GET /api/v1/events` è un flusso di eventi: prima
l'intero stato, poi ogni modifica a un lettore, a una playlist, a un brano o
alla cartwall, e i tempi di ciò che sta suonando alcune volte al secondo.

    curl -sN http://127.0.0.1:7380/api/v1/events

Una pagina web usa `EventSource`. I browser non possono inviare lì il token
come header, quindi va nell'indirizzo: `/api/v1/events?token=<token>`.

## OSC {#osc}

OSC è il protocollo abituale delle superfici di controllo, dei banchi luci e
dei software di controllo degli spettacoli. Attivalo con `remote.osc.enabled`.
Ascolta sulla porta UDP 7381 di questo computer. Per accettare pacchetti da
altri computer, imposta `remote.osc.bind` su `0.0.0.0` ed elenca i loro
indirizzi o sottoreti in `remote.osc.allowed_sources` (per esempio
`"192.168.1.0/24"`). OSC non ha una password, quindi tienilo su una rete di
studio fidata.

I lettori sono numerati 1, 2, 3… come compaiono sullo schermo. La colonna **Musica**
(il riempitivo musicale, vedi [Automazione](automation.md)) non ha un numero
e non fa parte del controllo remoto. I cart sono
numerati nella pagina mostrata.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

Una superficie che vuole mostrare lo stato (luci, nomi, conti alla rovescia)
si sottoscrive, e poi riceve ogni valore una volta e in seguito solo ciò che
cambia. Quando un lettore o un pulsante di cart sparisce (meno lettori, una
pagina più piccola), i suoi indirizzi ricevono una volta un valore vuoto, in
modo che la superficie li cancelli. Deve sottoscriversi di nuovo entro un
minuto (`subscription_ttl_secs`) per continuare a ricevere:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

Su un lettore la cui uscita Main è bit-perfect, i messaggi `/volume` vengono
rifiutati (il log lo annota) e il lettore invia `/volume-locked` 1 ai
sottoscrittori (0 altrimenti).

Un sottoscrittore può indicare qualsiasi porta del proprio indirizzo, e ne
vengono mantenuti fino a `max_subscribers`. Chiunque abbia il permesso di
inviare può quindi anche sottoscriversi. È un altro motivo per tenere OSC su
una rete fidata.

`oscsend` e `oscdump` fanno parte di liblo (`liblo-tools` su Debian e
Ubuntu). L'elenco completo degli indirizzi è nella
[documentazione tecnica](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

## Tutte le impostazioni {#all-settings}

| Impostazione | Predefinito | Significato |
|---|---|---|
| `remote.http.enabled` | `false` | Attiva l'API |
| `remote.http.bind` | `127.0.0.1` | Indirizzo su cui ascoltare (un indirizzo IP) |
| `remote.http.port` | `7380` | Porta (1024–65535) |
| `remote.http.token` | vuoto | Richiesto oltre questo computer; almeno 16 caratteri |
| `remote.http.cors_origins` | nessuna | Origini web autorizzate a chiamare l'API |
| `remote.http.request_timeout_ms` | `10000` | Il massimo che una richiesta può durare |
| `remote.http.max_body_bytes` | `65536` | Corpo di richiesta più grande |
| `remote.http.max_event_clients` | `16` | Flussi di eventi in tempo reale contemporanei |
| `remote.osc.enabled` | `false` | Attiva OSC |
| `remote.osc.bind` | `127.0.0.1` | Indirizzo su cui ascoltare |
| `remote.osc.port` | `7381` | Porta UDP (1024–65535) |
| `remote.osc.allowed_sources` | questo computer | Indirizzi o sottoreti di cui si accettano i pacchetti |
| `remote.osc.max_subscribers` | `16` | Sottoscrittori contemporanei |
| `remote.osc.subscription_ttl_secs` | `60` | Una sottoscrizione non rinnovata entro questo tempo termina |
| `remote.events.position_interval_ms` | `250` | Ogni quanto vengono pubblicati i tempi durante la riproduzione |

I valori fuori intervallo vengono corretti quando il file viene caricato, e
la correzione viene registrata nel log.
