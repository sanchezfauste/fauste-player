# Risoluzione dei problemi

## Nessun suono {#no-sound}

1. Apri **Impostazioni → Uscite audio** e premi **Prova Main** per il
   lettore. Se senti il tono, controlla il fader del volume del lettore.
2. Se non senti nulla, scegli un altro dispositivo o un'altra coppia di
   canali. Le modifiche alle uscite hanno effetto dopo un riavvio: premi
   **Riavvia ora** nelle Impostazioni.
3. Su Linux, preferisci **PipeWire** o **PulseAudio** in Impostazioni →
   Uscite audio → Sistema audio. Condividono la scheda audio con altri
   programmi. **ALSA** parla direttamente con la scheda e potrebbe trovarla
   occupata.
4. **JACK** risulta non disponibile («nessun dispositivo di uscita») quando
   non è in esecuzione alcun server JACK. Avvia il server (per esempio con
   QjackCtl) e riavvia l'applicazione. Imposta il server JACK sulla
   frequenza di campionamento delle Impostazioni (48 kHz per impostazione
   predefinita): JACK funziona a una sola frequenza per tutti i programmi.
5. **PipeWire** non è offerto dagli archivi scaricabili; questi raggiungono
   PipeWire tramite il suo servizio PulseAudio, che funziona allo stesso
   modo. È disponibile nelle build realizzate con la feature `pipewire`.

## Bit-perfect {#bit-perfect}

- **Il badge BP resta spento.** Controlla ogni condizione in
  [Uscita bit-perfect](bit-perfect.md#the-bp-badge): volume al 100 %, nessuna
  dissolvenza, nient'altro sulle stesse uscite, un file lossless che sia
  stato analizzato e un dispositivo che funzioni alla frequenza del file.
- **Un breve silenzio prima di un brano.** Il dispositivo bit-perfect è stato
  riaperto alla frequenza di campionamento del brano. Tieni la libreria a
  una sola frequenza per evitarlo.
- **Un brano suona ricampionato, e il log dice che il dispositivo è occupato
  (Linux).** Per cambiare frequenza, l'applicazione chiude il dispositivo e
  lo riapre. In quel momento il server audio (PipeWire) può prendersi la
  scheda. L'applicazione riprova alcune volte; se la scheda è ancora
  occupata, il brano suona alla frequenza attuale del dispositivo, e il
  brano successivo chiede di nuovo la sua frequenza. Per dare la scheda
  all'applicazione in via esclusiva, apri le impostazioni audio del sistema e
  imposta il profilo di quella scheda su **Off** (o **Pro Audio**), in modo
  che il server audio lasci in pace il suo dispositivo `hw:`. Il numero di
  tentativi e l'attesa tra l'uno e l'altro sono `tuning.device_busy_retries` e
  `tuning.device_busy_retry_ms` nel file di configurazione.
- **Il dispositivo suona, ma il badge BP resta spento (Windows o macOS).**
  L'accesso esclusivo è stato rifiutato, e il dispositivo suona in modo
  condiviso.
  - Windows: un altro programma potrebbe tenere il dispositivo in modo
    esclusivo, oppure il controllo esclusivo è disattivato nelle proprietà
    Avanzate del dispositivo.
  - macOS: un altro programma potrebbe tenere il dispositivo in hog mode,
    oppure il dispositivo offre le sue frequenze solo come intervallo
    continuo (la maggior parte delle interfacce elenca frequenze fisse).
- **Un dispositivo `hw:` non si può aprire (Linux).**
  - Un server audio potrebbe tenere la scheda. Fermalo, o imposta il server
    in modo che lasci in pace quella scheda, e riavvia l'applicazione.
  - Alcuni DAC USB accettano solo campioni a 24 bit impacchettati
    (`S24_3LE`), che la libreria audio non supporta. Usa quella scheda
    tramite `plughw:` (non bit-perfect).

### DSD {#dsd}

- **Un brano DSD suona convertito anche se il dispositivo è impostato su DoP
  o DSD nativo.** Il log dice perché («DSD converted to PCM» e il motivo) per
  queste cause: il volume del lettore non al 100 %, qualcos'altro che suona
  sul dispositivo, più di due canali, o un dispositivo che rifiuta la
  frequenza (il DoP richiede la frequenza DSD divisa per 16, per esempio
  176,4 kHz per il DSD64) o non ha un formato a 24 o 32 bit. Un brano non
  ancora analizzato viene convertito in silenzio, senza riga di log:
  analizzalo (Impostazioni → Analisi) e riproducilo di nuovo.
- **Solo il primo brano di un album DSD esce come DSD.** È l'impostazione di
  mixaggio predefinita: i brani che il lettore avvia da solo suonano
  convertiti. Scegli **Mantieni il DSD e silenzia le altre sorgenti** in
  Impostazioni → Uscite audio per mantenerli DSD. Vedi
  [DSD](bit-perfect.md#dsd).
- **L'intestazione mostra DSD ma il convertitore suona rumore o non si
  aggancia.** Il convertitore non riconosce il DoP (o il formato nativo).
  Riporta il dispositivo su **Converti in PCM**.
- **Un clic quando un brano DSD parte, si ferma o esce dal DSD.** Il
  convertitore ha bisogno di più silenzio DSD: aumenta **Silenzio DSD** (200
  ms per impostazione predefinita) in Impostazioni → Uscite audio, Avanzate.
- **Gli altri lettori o cart sono muti sul dispositivo.** Sta suonando un
  brano DSD con **Mantieni il DSD e silenzia le altre sorgenti**; il badge
  **Altre mute** è visibile. Tornano a suonare quando il brano finisce.

## Avviso «Uscita persa» {#output-lost-alert}

La barra di stato mostra **Uscita persa: &lt;dispositivo&gt;** quando un
dispositivo smette di rispondere. I lettori continuano a contare e a
mixare su un clock interno, così l'automazione non si blocca. Il dispositivo
viene ritentato ogni 2 secondi e riprende il controllo quando ritorna.
Ricollega il cavo o riaccendi l'interfaccia.

### «Uscita persa» che non scompare mai, con un'uscita `hw:` diretta {#output-lost-that-never-clears-with-a-direct-hw-output}

Una scheda audio usata tramite un'uscita ALSA `hw:` diretta (per esempio
un'uscita bit-perfect) è tenuta dal solo Fauste Player: il server audio
(PipeWire o PulseAudio) non può usarla contemporaneamente. Se un'altra
uscita passa per il dispositivo predefinito del server audio e quel
dispositivo predefinito è la stessa scheda, quell'uscita non parte mai e
resta **Uscita persa**. Il log dice una volta «output device opened but never
started».

Usa un solo percorso per scheda: instrada ogni uscita di quella scheda
attraverso lo stesso dispositivo `hw:` (con canali diversi, se serve), oppure
scegli un'altra scheda come uscita predefinita del server audio nelle
impostazioni audio del tuo sistema.

## Un brano mostra un'icona di avviso o un file con una croce {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

Il file manca (spostato, eliminato, smontato: un file con una croce) oppure
non si può decodificare (un segno di avviso). I lettori lo saltano. Passa il
puntatore sulla riga per vedere quale dei due, e il percorso del file.

Un file mancante viene cercato di nuovo ogni 30 secondi
(`tuning.missing_recheck_ms` nel file di configurazione): quando l'unità
viene montata o il file viene rimesso a posto, il brano torna riproducibile
da solo. Un file che non si può decodificare viene ricontrollato da solo con
lo stesso timer, in base alla dimensione e alla data di modifica: non viene
decodificato di nuovo a meno che una delle due sia cambiata, per esempio
quando finisce una copia. Per controllarlo subito usa **Rianalizza** nel menu
della sua riga, oppure **Impostazioni → Analisi → Rianalizza tutti i brani**
per l'intera libreria.

## Interruzioni audio {#audio-dropouts}

La barra di stato avvisa per 5 secondi dopo ogni interruzione che
l'applicazione rileva: **P1: interruzioni audio (3)** quando la
decodifica di un lettore non ha tenuto il passo con il disco (il conteggio è
per il brano in riproduzione), e **&lt;dispositivo&gt;: interruzioni del
dispositivo audio (2)** quando il dispositivo di uscita ha mancato una
scadenza (uno xrun). Anche il log registra ognuna, al massimo una riga ogni
10 secondi per tipo, con quante sono avvenute. Non tutti i sistemi audio
segnalano gli xrun (PulseAudio no; la modalità esclusiva di Windows no).


- Aumenta la **dimensione del buffer** nelle Impostazioni (e premi **Riavvia
  ora**).
- Su Linux, consenti la schedulazione in tempo reale. L'applicazione la
  chiede al sistema tramite rtkit (D-Bus). Funziona anche l'appartenenza al
  gruppo `audio` con un limite `rtprio`.
- Evita le unità di rete per la musica che va in onda.

## «L'interfaccia ha riscontrato un errore» {#the-interface-hit-an-error}

È stato intercettato un errore di disegno. L'audio non ne risente. Premi
**Riavvia l'interfaccia**. Segnalalo, per favore, insieme ai log.

## Log e report dei crash {#logs-and-crash-reports}

Vedi [Dati e backup](data-and-backups.md) per la cartella dei log. C'è un
file di log al giorno, e gli ultimi 14 vengono conservati. I report dei crash
vengono salvati come `crash-<time>.txt`. Imposta `RUST_LOG=debug` nell'ambiente
per maggiori dettagli. Allega entrambi i file quando segnali un bug.
