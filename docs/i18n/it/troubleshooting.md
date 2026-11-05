# Risoluzione dei problemi

## Nessun suono {#no-sound}

1. Apri **Impostazioni → Uscite audio** e premi **Prova Main** per il
   lettore. Se senti il tono, controlla il fader del volume del lettore.
2. Se non senti nulla, scegli un altro dispositivo o un'altra coppia di
   canali. Una modifica alle uscite si applica quando su di esse non suona più
   nulla; **Applica ora** nelle Impostazioni la applica subito.
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
  [Uscita bit-perfect](bit-perfect.md#the-bp-badge): nessuna dissolvenza,
  nient'altro sulle stesse uscite, un file lossless che sia stato analizzato
  e un dispositivo che funzioni alla frequenza del file.
- **Un breve silenzio prima di un brano.** Il dispositivo bit-perfect è stato
  riaperto alla frequenza di campionamento del brano. Succede solo quando la
  frequenza cambia: i brani alla stessa frequenza si susseguono senza pausa.
  Tieni la libreria a una sola frequenza per evitarlo. Vedi
  [Cosa costa](bit-perfect.md#what-it-costs).
- **I brani non si mixano, e Play non fa dissolvenze.** Su un lettore la cui
  uscita Main è bit-perfect, ogni brano suona fino alla fine e poi parte il
  successivo, e Play / Successivo e Precedente cambiano brano subito: su un
  lettore del genere non ci sono dissolvenze incrociate né segue. Disattiva il
  bit-perfect per il dispositivo, o sposta il lettore su un'altra uscita, per
  tornare a mixare. Vedi [Cosa costa](bit-perfect.md#what-it-costs).
- **Il fader del volume non si muove.** L'uscita Main del lettore è
  bit-perfect, il che mantiene il volume al 100 %; lo dice il tooltip del
  fader. Neanche i fader MIDI e l'API remota possono muoverlo. Vedi
  [Cosa costa](bit-perfect.md#what-it-costs).
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

Un WAV o FLAC che risulta illeggibile, o che suonerebbe come rumore forte,
può essere un'estrazione di un CD DTS: il file contiene un bitstream DTS
invece dell'audio. L'applicazione lo rifiuta (il log dice «encoded bitstream
stored as PCM») e non decodifica il DTS. Convertilo prima in normale PCM con
un altro strumento.

## Una cartella sorvegliata mostra un avviso {#a-watched-folder-shows-a-warning}

Una categoria che sorveglia una cartella mostra ⟳, di un altro colore quando
la cartella non viene seguita; passaci sopra il puntatore per il motivo. Vedi
[Cartelle sorvegliate](library.md#watched-folders).

- **Non trovata o non leggibile**: l'unità o la condivisione non è montata o
  non è raggiungibile. Anche una cartella vuota che dovrebbe contenere brani
  della libreria conta come non trovata, perché una condivisione smontata
  spesso sembra così. Nulla viene segnato o rimosso; la cartella viene
  cercata di nuovo ogni 30 secondi e analizzata quando compare. Se hai
  svuotato la cartella di proposito, elimina i suoi brani selezionandoli
  nella tabella della libreria e scegliendo **Rimuovi dalla libreria…**,
  oppure scegliendo **Rianalizza** su di essi, il che segna come mancanti
  quelli il cui file è sparito così che **Rimuovi i brani mancanti…** possa
  rimuoverli.
- **Le modifiche non vengono seguite**: il sistema non vuole sorvegliarla,
  di solito una cartella con moltissime sottocartelle. La cartella viene
  comunque analizzata a ogni avvio e a ogni nuovo tentativo, quindi i nuovi
  file compaiono allora, non subito. Su Linux, un albero di cartelle più
  grande del limite di controlli per utente richiede un
  `fs.inotify.max_user_watches` più alto (per esempio `sudo sysctl
  fs.inotify.max_user_watches=524288`; rendilo permanente in
  `/etc/sysctl.d/`).
- **Non è un percorso completo**: la cartella memorizzata è stata modificata
  a mano in un percorso non completo. Scegli **Cambia cartella
  sorvegliata…** per sceglierla di nuovo.
- **Una condivisione di rete non mostra alcun avviso, ma i file aggiunti da
  un altro computer non compaiono**: su una condivisione di rete (SMB/CIFS,
  NFS) il sistema di solito accetta il controllo ma segnala solo le modifiche
  fatte da questo computer. Le modifiche fatte altrove si vedono al prossimo
  avvio, non mentre l'applicazione è in esecuzione.
- **Una cartella con migliaia di file** richiede un po' di tempo per essere
  analizzata all'avvio; l'avanzamento dell'analisi lo mostra.
- I brani il cui file ha lasciato una cartella sorvegliata restano nella
  libreria segnati come mancanti. Rimuovili con **Rimuovi i brani
  mancanti…** nel menu della categoria.

## Interruzioni audio {#audio-dropouts}

La barra di stato avvisa per 5 secondi dopo ogni interruzione che
l'applicazione rileva: **P1: interruzioni audio (3)** quando la
decodifica di un lettore non ha tenuto il passo con il disco (il conteggio è
per il brano in riproduzione), e **&lt;dispositivo&gt;: interruzioni del
dispositivo audio (2)** quando il dispositivo di uscita ha mancato una
scadenza (uno xrun). Anche il log registra ognuna, al massimo una riga ogni
10 secondi per tipo, con quante sono avvenute. Non tutti i sistemi audio
segnalano gli xrun (PulseAudio no; la modalità esclusiva di Windows no).


- Aumenta la **dimensione del buffer** nelle Impostazioni (si applica quando
  su quel dispositivo non suona più nulla, oppure subito con **Applica ora**).
- Attraverso un server audio (PulseAudio, il dispositivo ALSA `default`),
  ogni uscita chiede già almeno 50 ms qualunque sia la dimensione del buffer,
  così un computer occupato non si sente; le Impostazioni lo mostrano accanto
  alla latenza. Su un computer vecchio o molto carico, alza `shared_buffer_ms`
  nella sezione `outputs` del file di configurazione (per esempio a 100). Il
  sistema tiene in coda circa due di questi blocchi, quindi i pulsanti dei
  cart e di play suonano poi circa il doppio dell'aumento più tardi. Per
  l'uscita in onda, la voce hardware del dispositivo (`hw:` su Linux) evita
  del tutto il server audio.
- Su Linux, consenti la schedulazione in tempo reale. L'applicazione la
  chiede al sistema tramite rtkit (D-Bus), e il log registra un rifiuto.
  Molti desktop la rifiutano alle applicazioni.
- Evita le unità di rete per la musica che va in onda.

## L'interfaccia è lenta su un computer vecchio {#the-interface-is-slow-on-an-old-computer}

Mentre qualcosa suona, l'interfaccia disegna al massimo 30 fotogrammi al
secondo (`max_fps` nella sezione `ui` del file di configurazione), e i clic e
i tasti ricevono risposta subito. Su un computer vecchio o occupato abbassalo
(per esempio a 20) per lasciare più processore all'audio; 0 disegna ogni
fotogramma che mostra lo schermo. I meter misurano lo stesso a qualsiasi
frequenza; vengono solo disegnati meno spesso.

## Non si possono rilasciare file su Linux {#files-cannot-be-dropped-on-linux}

La finestra accetta file trascinati da un file manager solo quando gira su
X11. Per impostazione predefinita lo fa, anche su un desktop Wayland
(tramite XWayland, che ogni desktop ha). Se `ui.window_system` in
`config.json` è impostato su `"Wayland"`, la finestra è una finestra Wayland
nativa, che può apparire più nitida con la scala frazionaria dello schermo,
ma non vi si possono rilasciare file: usa invece **+ Aggiungi**. Rimettilo su
`"Auto"` (il valore predefinito) o `"X11"` e riavvia Fauste Player.

## Un controller MIDI non risponde {#a-midi-controller-does-not-respond}

Se **Impostazioni → MIDI** mostra il controller come connesso ma i suoi
pulsanti e fader non fanno nulla, premi lì **Riconnetti le porte**: il
programma chiude le porte MIDI e le riapre, come scollegare e ricollegare il
controller. Se non basta, scollega il controller e ricollegalo.

In entrambi i casi, il log dice cosa è successo. Elenca le porte MIDI ogni
volta che cambiano, ogni porta aperta e chiusa (con il motivo), una porta che
non si può aprire (una volta, finché non si apre o sparisce), il primo
messaggio ricevuto su ogni connessione, il primo messaggio ignorato su ogni
connessione con il motivo (nessuna associazione, un fader che non ha ancora
raggiunto il volume, un'azione che ora non può agire) e il primo messaggio
LED fallito. Un controller connesso che non registra mai un primo messaggio
non sta inviando nulla che il programma riceva. Per ogni messaggio ignorato e
ogni nuovo tentativo fallito, avvia il programma con
`RUST_LOG=info,fp_control=debug`.

## «L'interfaccia ha riscontrato un errore» {#the-interface-hit-an-error}

È stato intercettato un errore di disegno. L'audio non ne risente. Premi
**Riavvia l'interfaccia**. Segnalalo, per favore, insieme ai log.

## Log e report dei crash {#logs-and-crash-reports}

Vedi [Dati e backup](data-and-backups.md) per la cartella dei log. C'è un
file di log al giorno. Le righe identiche che si ripetono una dopo l'altra
vengono scritte una sola volta, seguite da una riga come `previous message
repeated 12 times` (durante una lunga serie di ripetizioni quella riga viene
scritta ogni minuto). Le righe che differiscono, per esempio perché portano un
contatore, non vengono compattate. I messaggi di connessione di routine del
client PulseAudio sono omessi per impostazione predefinita. Imposta
`RUST_LOG=debug` nell'ambiente per maggiori dettagli (sostituisce il filtro
predefinito, `info,pulseaudio=warn`).

Il log è limitato: il file di un giorno smette di crescere a 10 MiB (la sua
ultima riga lo dice; il resto di quel giorno non viene registrato, ed è ciò a
cui porta un diluvio di righe diverse) e vengono conservati gli ultimi 14
file. Si cambiano nel file di configurazione con `limits.max_log_file_mb` (da
1 a 1024), `limits.max_log_files` (da 2 a 365) e
`limits.log_repeat_summary_secs` (da 1 a 3600, predefinito 60, ogni quanto
viene segnalata una lunga serie di ripetizioni); si applicano al prossimo
avvio. I report dei crash vengono salvati come `crash-<time>.txt` nella stessa
cartella e sono limitati separatamente (`limits.max_crash_reports` per
esecuzione). Allega entrambi i file quando segnali un bug.
