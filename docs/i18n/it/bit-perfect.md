# Uscita bit-perfect

Un dispositivo **bit-perfect** riceve i campioni di ogni file esattamente
come sono nel file: stessa frequenza di campionamento, stessi valori, senza
ricampionamento, variazioni di volume o mixaggio. È utile per le catene di
monitoraggio e i collegamenti digitali, dove va evitata qualsiasi
elaborazione sul computer.

## Rendere bit-perfect un dispositivo {#setting-a-device-bit-perfect}

1. In **Impostazioni → Uscite audio**, scegli esplicitamente il dispositivo
   per l'uscita Main di un lettore (o della cartwall). Un lettore lasciato
   sul predefinito di sistema non può essere reso bit-perfect. Un
   dispositivo che nessuna uscita usa più perde il suo interruttore
   bit-perfect e la sua modalità DSD alla prossima partenza
   dell'applicazione.
2. Scegli **Avanzate** in cima alla sezione. Sotto **Impostazioni per
   dispositivo**, attiva **Bit-perfect** accanto al dispositivo.
3. Riavvia l'applicazione.

Il dispositivo parte allora alla propria frequenza di campionamento se gliene
hai data una nello stesso punto, altrimenti alla frequenza di campionamento
globale, e da lì segue ogni file.

L'interruttore è disattivato quando il dispositivo non può dare l'accesso
esclusivo.
- **Linux:** scegli un dispositivo ALSA il cui nome inizia con `hw:`. È la
  scheda audio stessa. PulseAudio, PipeWire, JACK e i dispositivi ALSA
  `default` o `plughw:` mixano o convertono, quindi non sono mai
  bit-perfect.
- **Windows:** scegli il dispositivo nel sistema **WASAPI**. Viene aperto in
  modalità esclusiva.
  - Nelle impostazioni audio di Windows, le proprietà **Avanzate** del
    dispositivo devono avere attivata l'opzione *Consenti alle applicazioni
    di assumere il controllo esclusivo del dispositivo* (è attiva per
    impostazione predefinita).
  - Mentre suona, nessun altro programma può usare il dispositivo.
- **macOS:** scegli il dispositivo in **Core Audio**. Viene aperto in hog
  mode.
  - La frequenza di campionamento del dispositivo viene impostata su quella
    del brano, e il suo formato sul formato intero più ampio che offre a
    quella frequenza (le impostazioni che mostra Configurazione Audio MIDI).
  - Vengono restituite quando l'applicazione smette di usare il dispositivo.
  - Due dispositivi con esattamente lo stesso nome non possono essere resi
    bit-perfect.

## Cosa succede su un dispositivo bit-perfect {#what-happens-on-a-bit-perfect-device}

- **Accesso esclusivo.** Nient'altro sul computer può suonare sul
  dispositivo mentre l'applicazione lo usa. Se l'accesso esclusivo viene
  rifiutato, il dispositivo suona comunque, condiviso, e il badge BP resta
  spento.
- **La frequenza segue il file.** Quando sul dispositivo non suona nulla e
  parte un brano a un'altra frequenza di campionamento, il dispositivo viene
  riaperto a quella frequenza.
  - Succede quando riproduci un brano, riprendi un brano caricato in pausa,
    preascolti o lanci un cart. I brani che sono solo in attesa (il brano
    successivo di ogni lettore) vengono preparati di nuovo alla nuova
    frequenza.
  - La riapertura richiede il tempo che serve al dispositivo per avviarsi
    (di solito alcune decine di millisecondi). L'avvio avviene altrettanto
    più tardi.
  - Finché sul dispositivo suona qualcosa, la frequenza non cambia mai. Un
    brano a un'altra frequenza che parte in quel momento (per esempio un
    brano a 48 kHz in cui si mixa dopo uno a 44,1 kHz, o un brano avviato
    mentre un altro lettore o un cart suona sullo stesso dispositivo) viene
    convertito per tutta la sua durata, e non è bit-perfect.
  - Se il dispositivo rifiuta una frequenza, mantiene la precedente e il
    brano viene convertito.
- **Nessuna elaborazione, quando nulla la richiede.** I campioni passano
  inalterati finché valgono tutte queste condizioni:
  - il volume del lettore è al 100 %;
  - non è in corso alcuna dissolvenza;
  - nient'altro suona sulle stesse uscite (un altro lettore, un cart, un
    tono di prova).

## DSD {#dsd}

Un file DSD normalmente viene riprodotto convertito in PCM, come qualsiasi
altro file. Un dispositivo bit-perfect può invece ricevere il flusso DSD
inalterato.

**Le tre modalità.** Nella vista Avanzate di Impostazioni → Uscite audio,
ogni dispositivo che un'uscita usa ha una scelta **DSD** sotto il suo
interruttore bit-perfect:
- **Converti in PCM** (il valore predefinito): il DSD viene convertito,
  come su qualsiasi altro dispositivo.
- **DoP** (DSD over PCM): i bit DSD viaggiano dentro campioni PCM a 24 bit,
  che la maggior parte dei convertitori compatibili DSD riconosce.
  Funziona su ogni sistema.
- **DSD nativo** (solo Linux): DSD grezzo, per i dispositivi ALSA `hw:` il
  cui driver indica un formato di campione DSD.

Vengono offerte solo le modalità che il dispositivo può accettare, e una
riga sotto la scelta dice perché le altre no: il dispositivo non è
collegato, il bit-perfect è disattivato, il dispositivo non si può aprire in
esclusiva, il DSD nativo richiede Linux, oppure il dispositivo non accetta il
DSD nativo. Una modalità salvata per un dispositivo che ora non può
accettarla viene mostrata come PCM, che è ciò che suona; la modalità salvata
torna quando il dispositivo potrà accettarla di nuovo. Cambiare una
modalità, l'impostazione del mixaggio o il silenzio DSD richiede un riavvio,
come le altre impostazioni delle uscite.

**Quando il DSD esce inalterato.** Tutte queste condizioni devono valere
quando il brano parte:
- il dispositivo è bit-perfect, con accesso esclusivo, e la sua modalità è
  DoP o DSD nativo;
- il brano è DSD (DSF o DFF), mono o stereo, ed è stato analizzato (è così
  che si conosce la sua frequenza DSD);
- il volume del lettore è al 100 %;
- nient'altro suona sul dispositivo (un altro lettore, un cart, un tono di
  prova);
- il dispositivo accetta il flusso. Il DoP richiede una frequenza del
  dispositivo pari alla frequenza DSD divisa per 16 (176,4 kHz per il DSD64,
  352,8 kHz per il DSD128, 705,6 kHz per il DSD256) e un formato a 24 o 32
  bit. Il DSD nativo richiede un dispositivo che accetti il formato DSD a
  quella frequenza.

Quando il DSD nativo finisce, il dispositivo torna al PCM alla frequenza che
aveva prima del brano DSD, perché molti convertitori accettano il DSD
nativo a frequenze alle quali non possono riprodurre PCM (nessun convertitore
riproduce PCM alla frequenza a cui funziona il DSD512). Un brano che
prosegue come PCM mantiene la frequenza del flusso DSD quando il dispositivo
la accetta come PCM, altrimenti continua alla frequenza precedente da dove
si trovava, come tutto il resto che suona su quel dispositivo; una
dissolvenza in corso lì termina subito. Il dispositivo non resta mai su una
frequenza che rifiuta: se nessuna frequenza si apre (per esempio, il
dispositivo è stato scollegato in quel momento), l'uscita è persa finché il
nuovo tentativo automatico non la riapre alla frequenza precedente.

Altrimenti il brano viene convertito in PCM e il log dice perché (per
esempio «something else plays on the device» o «the device refused 705600
Hz»). Il preascolto e i cart vengono sempre convertiti.

Mentre il DSD esce inalterato:
- il badge nell'intestazione mostra **DSD** invece di **BP**;
- i meter mostrano il livello della conversione PCM dello stesso brano,
  quindi funzionano come al solito;
- il volume deve restare al 100 %: lo dice il tooltip del fader. Muoverlo
  porta il brano in PCM (vedi sotto);
- Stop e Stop in dissolvenza fermano il brano subito, senza dissolvenza,
  perché un flusso DSD non può essere dissolto. Premere Play su un altro
  brano mentre suona lo interrompe allo stesso modo invece di fare una
  dissolvenza incrociata;
- anche la pausa e la ripresa agiscono subito, senza rampa.

**Silenzio agli estremi.** Ogni avvio, fine e passaggio a PCM invia prima
silenzio DSD (200 ms per impostazione predefinita), perché il convertitore
si agganci senza clic. L'eccezione è un brano DSD che continua un flusso
dello stesso tipo e della stessa frequenza DSD il cui silenzio è ancora in
corso: il convertitore è ancora agganciato, quindi parte senza silenzio
aggiuntivo. Un brano quindi parte altrettanto più tardi, e un passaggio a
PCM lascia uno spazio di quella durata. Si chiama **Silenzio DSD** in
Impostazioni → Uscite audio, Avanzate (da 0 a 2000 ms).

**Quando un'altra sorgente ha bisogno del dispositivo.** **Quando un'altra
sorgente ha bisogno di un'uscita DSD** in Impostazioni → Uscite audio sceglie
cosa succede quando un altro lettore, un cart o un tono di prova parte sullo
stesso dispositivo (muovere il fader del lettore stesso è l'eccezione: porta
sempre il brano in PCM):
- **Prosegui il brano DSD in PCM** (il valore predefinito). Il flusso passa
  al PCM dopo il silenzio DSD, e il brano prosegue, convertito, da dove era.
  Lo stesso accade al brano che segue da solo (vedi sotto).
- **Mantieni il DSD e silenzia le altre sorgenti.** Nulla interrompe il
  flusso DSD. Le altre sorgenti instradate al dispositivo restano mute
  finché il brano DSD non finisce, e nel frattempo il lettore mostra un
  badge **Altre mute**. Il brano successivo del lettore stesso non si
  sovrappone: parte quando il brano DSD finisce, senza dissolvenza
  incrociata né segue. Un brano PCM aspetta il silenzio DSD; un brano DSD
  dello stesso tipo e della stessa frequenza DSD continua il flusso senza
  di esso. Muovere il fader porta comunque il brano in PCM.

**Un album non resta DSD con l'impostazione predefinita.** Con *Prosegui il
brano DSD in PCM*, esce come DSD solo un brano DSD che parte su un
dispositivo inattivo. I brani che il lettore avvia da solo dopo (alla fine
di un brano, in un segue o in una dissolvenza incrociata) partono da un
precaricamento, che è sempre PCM, quindi il dispositivo passa al PCM e
suonano convertiti. Un brano che avvii tu (Play, doppio clic) esce di nuovo
come DSD quando il dispositivo è inattivo o il flusso DSD precedente è
ancora nel suo silenzio alla stessa frequenza DSD. Per mantenere come DSD un
intero album DSD, scegli *Mantieni il DSD e silenzia le altre sorgenti*.
Allora ogni brano del lettore esce come DSD, e il successivo parte quando
finisce il precedente.

Se il dispositivo viene perso mentre suona il DSD e ritorna senza poterlo
trasportare (per esempio senza accesso esclusivo), il brano prosegue come
PCM.

## Il badge BP {#the-bp-badge}

Il badge **BP** nell'intestazione del lettore si accende finché il brano
attuale arriva inalterato al suo dispositivo Main. Tutte queste condizioni
devono valere:

- il dispositivo è bit-perfect e aperto con accesso esclusivo;
- il dispositivo funziona alla frequenza di campionamento del brano;
- il brano è PCM intero lossless (WAV, AIFF, FLAC, ALAC, WavPack o Monkey's
  Audio), mono o stereo, e al massimo a 24 bit, e il formato del dispositivo
  contiene la dimensione del suo campione (un file a 24 bit su un
  dispositivo a 16 bit non è bit-perfect). Il DSD viene convertito, quindi
  non accende mai BP; quando esce inalterato (vedi [DSD](#dsd)) il badge
  mostra invece **DSD**;
- il brano è stato analizzato, perché è così che si conoscono la sua
  frequenza e la dimensione del campione. I brani analizzati da una versione
  precedente ottengono il loro formato una volta analizzati di nuovo (l'avviso
  all'avvio, o Impostazioni → Analisi), o appena un lettore li mostra o un
  cart li contiene;
- il volume è al 100 %, non è in corso alcuna dissolvenza e nient'altro suona
  sulle stesse uscite.

Alcuni file non vengono mai mostrati come bit-perfect:
- **File con perdita** (MP3, AAC, Ogg Vorbis, Opus): i loro campioni
  decodificati non sono i valori interi che un dispositivo accetta.
- **File oltre i 24 bit:** il mixer lavora in virgola mobile a 32 bit, che
  rappresenta esattamente 24 bit.
- **File con più di due canali:** vengono mixati in stereo.

## Verificarlo da sé {#checking-it-yourself}

Per verificare una catena da un capo all'altro:

1. Collega l'uscita digitale del dispositivo (S/PDIF, AES o loopback USB) a
   un registratore che catturi in modo bit-esatto.
2. Riproduci un file di prova lossless al 100 % con nient'altro in
   riproduzione.
3. Registralo.
4. Confronta la registrazione con il file. Per esempio, con SoX, inverti
   uno dei due e mixali: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`
   dopo aver allineato i loro inizi. Ogni campione della differenza deve
   essere zero.

I test automatici del progetto verificano la stessa proprietà all'interno
dell'applicazione, su un dispositivo simulato.

### DSD su un convertitore reale {#dsd-on-a-real-converter}

I test automatici verificano il DSD solo su dispositivi simulati. Il DoP e il
DSD nativo non sono stati provati dal progetto su un convertitore reale. Per
verificarne uno:
1. Imposta il dispositivo su **DoP** (o su **DSD nativo** su Linux),
   riavvia e riproduci un file DSD al 100 % con nient'altro in riproduzione.
   L'intestazione deve mostrare **DSD**, e il display del convertitore
   dovrebbe mostrare la frequenza DSD (per esempio DSD64) invece di una
   frequenza PCM. Un convertitore che mostra una frequenza PCM o riproduce
   rumore non riconosce il flusso: torna a **Converti in PCM**.
2. Ascolta se c'è un clic o una raffica di rumore all'inizio, allo Stop, alla
   fine del brano e quando muovi il fader. Un clic significa che il
   convertitore ha bisogno di un **Silenzio DSD** più lungo (Impostazioni →
   Uscite audio, Avanzate).
3. Avvia un cart o un altro lettore sullo stesso dispositivo, una volta con
   ciascuna impostazione di mixaggio, e verifica il comportamento descritto
   sopra.
4. Su Linux, per verificare il DSD nativo senza l'applicazione, esegui
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   Apre il dispositivo in DSD nativo a DSD64 e riproduce un secondo di
   silenzio DSD. Deve passare, e il convertitore dovrebbe agganciarsi a
   DSD64.
5. Con file DSD in `test-music/`, `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   li riproduce attraverso il motore su un dispositivo simulato e confronta
   le parole con i byte del file (vedi [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
