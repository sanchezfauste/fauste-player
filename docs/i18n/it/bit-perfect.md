# Uscita bit-perfect

Un dispositivo **bit-perfect** riceve i campioni di ogni file esattamente
come sono nel file: stessa frequenza di campionamento, stessi valori, senza
ricampionamento, variazioni di volume o mixaggio.

L'uscita bit-perfect è una **modalità speciale per l'ascolto audiofilo**:
ascoltare un archivio, alimentare una catena hi-fi o di monitoraggio,
verificare un collegamento digitale, ovunque vada evitata qualsiasi
elaborazione sul computer. È disattivata per impostazione predefinita e non è
pensata per un programma musicale mixato. Leggi [Cosa costa](#what-it-costs)
prima di attivarla.

## Cosa costa {#what-it-costs}

Per mantenere ogni brano intatto, un lettore la cui uscita Main è su un
dispositivo bit-perfect suona in modo diverso dagli altri:

- **Nessuna dissolvenza incrociata e nessun segue.** Ogni brano suona fino
  alla fine (il suo cue-out, o la fine del file con **Usa cue-in e cue-out**
  disattivato), poi parte il successivo. Il **Mix automatico al punto MIX** e
  il punto MIX non vengono usati: il marker MIX è disegnato attenuato e il
  badge **Mix** non si accende mai. **Play / Successivo** e **Precedente**
  mentre un brano è in onda cambiano brano subito, con un breve stop senza
  clic, invece di fare una dissolvenza.
- **Un breve silenzio quando cambia la frequenza di campionamento.** Tra due
  brani le cui frequenze differiscono, quando nient'altro suona sul
  dispositivo, il dispositivo prima riproduce ciò che ha ancora nel buffer (la
  fine del brano), poi viene riaperto alla nuova frequenza, il che lascia un
  breve silenzio (di solito alcune decine di millisecondi, di più su alcuni
  dispositivi). I brani alla stessa frequenza si susseguono senza pausa e
  senza sovrapposizione. Quando la frequenza non può seguire (vedi
  [Cosa succede su un dispositivo bit-perfect](#what-happens-on-a-bit-perfect-device)),
  il brano successivo parte subito, convertito alla frequenza del
  dispositivo.
- **Nessuna normalizzazione del loudness.** Un brano su un dispositivo
  bit-perfect suona sempre a guadagno unitario, anche con la
  [normalizzazione del loudness](loudness.md#bit-perfect-outputs) attiva:
  l'intestazione legge **0.0 dB** in ambra, e il suo tooltip ne spiega il
  motivo. Gli altri lettori e cart vengono normalizzati come al solito.
- **Il volume è mantenuto al 100 %.** Il fader del volume del lettore è
  bloccato in cima: sullo schermo, sui controller [MIDI](midi.md) e tramite
  l'[API remota e OSC](remote-control.md). Va al 100 % quando il bit-perfect
  viene attivato per il dispositivo, e vi resta quando viene disattivato.
- **Cosa funziona ancora:** **Stop in dissolvenza** (sfuma nella durata dello
  stop in dissolvenza), **Pausa**, **Stop**, **Riavvia**, i segni di ripetizione e di
  stop dopo, le modalità Single e Continua, il Cue (preascolto, che non ha un
  fader e non ne è toccato), i cart, l'[uscita DSD](#dsd) e ogni lettore il
  cui Main è su un altro dispositivo, che mixa come al solito. Un cart può
  suonare sullo stesso dispositivo: finché suona, il brano successivo del
  lettore parte alla frequenza attuale del dispositivo, convertito (quindi non
  è bit-perfect), e non viene inserito alcun silenzio.
- **Quando usarla.** Attivala quando ogni brano deve arrivare al convertitore
  intatto. Tienila disattivata per un programma in onda che richiede
  transizioni mixate o variazioni di livello. La modalità si imposta per
  dispositivo, quindi i lettori mixati possono restare su un altro
  dispositivo.

## Rendere bit-perfect un dispositivo {#setting-a-device-bit-perfect}

1. In **Impostazioni → Uscite audio**, scegli esplicitamente il dispositivo
   per l'uscita Main di un lettore (o della cartwall). Un lettore lasciato
   sul predefinito di sistema non può essere reso bit-perfect. Un
   dispositivo che nessuna uscita usa più perde il suo interruttore
   bit-perfect e la sua modalità DSD alla prossima partenza
   dell'applicazione.
2. Scegli **Avanzate** in cima alla sezione. Sotto **Impostazioni per
   dispositivo**, attiva **Bit-perfect** accanto al dispositivo.
3. Il dispositivo diventa bit-perfect appena non vi suona più nulla (vedi
   [Impostazioni in sospeso](settings.md#settings-pending)).

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
- **La frequenza segue il file.** Il dispositivo viene riaperto alla
  frequenza di campionamento di un brano quando il brano parte:
  - **Dal silenzio:** riproduci un brano, riprendi un brano caricato in
    pausa, preascolti o lanci un cart mentre sul dispositivo non suona nulla.
    I brani che sono solo in attesa (il brano successivo di ogni lettore)
    vengono preparati di nuovo alla nuova frequenza.
  - **Tra due brani di un lettore:** il brano in onda si ferma (alla sua
    fine, o subito per **Play / Successivo** e **Precedente**), il
    dispositivo si riapre e parte il brano successivo. È il breve silenzio di
    [Cosa costa](#what-it-costs).
  - Il dispositivo viene riaperto solo dopo aver riprodotto tutto ciò che
    aveva nel buffer, così la fine del brano precedente non viene mai
    tagliata. Poi la riapertura richiede il tempo che serve al dispositivo
    per avviarsi. Insieme sono di solito alcune decine di millisecondi;
    l'avvio avviene altrettanto più tardi. L'attesa del buffer non è mai
    più lunga di mezzo secondo (`tuning.play_out_max_ms` nel file di
    configurazione, da 0 a 2000 ms), qualunque cosa indichi il dispositivo.
- **Quando la frequenza non può seguire.** Il dispositivo mantiene la sua
  frequenza, e un brano a un'altra frequenza viene convertito per tutta la sua
  durata (non è bit-perfect), quando:
  - qualcos'altro suona sul dispositivo quando il brano parte: un altro
    lettore (o uno in pausa con un brano), un cart o un preascolto;
  - il dispositivo ha già rifiutato quella frequenza; mantiene la precedente;
  - il brano non è ancora stato analizzato, quindi la sua frequenza non è
    nota.

  Il passaggio tra i due brani resta comunque senza dissolvenza incrociata.
- **Nessuna elaborazione, quando nulla la richiede.** I campioni passano
  inalterati finché non è in corso alcuna dissolvenza (Stop in dissolvenza, le
  rampe di pausa e ripresa, il breve stop senza clic di un brano) e
  nient'altro suona sulle stesse uscite (un altro lettore, un cart, un tono
  di prova). Il volume non conta: un'uscita bit-perfect lo mantiene al 100 %.

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
modalità, l'impostazione del mixaggio o il silenzio DSD si applica quando sul
dispositivo non suona più nulla, come le altre impostazioni delle uscite.

**Quando il DSD esce inalterato.** Tutte queste condizioni devono valere
quando il brano parte:
- il dispositivo è bit-perfect, con accesso esclusivo, e la sua modalità è
  DoP o DSD nativo;
- il brano è DSD (DSF o DFF), mono o stereo, ed è stato analizzato (è così
  che si conosce la sua frequenza DSD);
- nient'altro suona sul dispositivo (un altro lettore, un cart, un tono di
  prova);
- il brano non ha alcun guadagno di
  [normalizzazione del loudness](loudness.md#bit-perfect-outputs): uno
  caricato (per esempio in pausa) con un guadagno prima che il dispositivo
  diventasse bit-perfect lo mantiene, quindi suona convertito, senza **DSD**
  né **BP**;
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
- Stop e Stop in dissolvenza fermano il brano subito, senza dissolvenza,
  perché un flusso DSD non può essere dissolto;
- anche la pausa e la ripresa agiscono subito, senza rampa.

**Silenzio agli estremi.** Ogni avvio, fine e passaggio a PCM invia prima
silenzio DSD (200 ms per impostazione predefinita), perché il convertitore
si agganci senza clic. L'eccezione è un brano DSD che continua un flusso
dello stesso tipo e della stessa frequenza DSD il cui silenzio è ancora in
corso: il convertitore è ancora agganciato, quindi parte senza silenzio
aggiuntivo. Un brano quindi parte altrettanto più tardi, e un passaggio a
PCM lascia uno spazio di quella durata (anche un dispositivo DSD nativo
riproduce ciò che ha nel buffer prima di essere riaperto come PCM, alcuni
millisecondi in più). Si chiama **Silenzio DSD** in
Impostazioni → Uscite audio, Avanzate (da 0 a 2000 ms).

**Quando un'altra sorgente ha bisogno del dispositivo.** **Quando un'altra
sorgente ha bisogno di un'uscita DSD** in Impostazioni → Uscite audio sceglie
cosa succede quando un altro lettore, un cart o un tono di prova parte sullo
stesso dispositivo:
- **Prosegui il brano DSD in PCM** (il valore predefinito). Il flusso passa
  al PCM dopo il silenzio DSD, e il brano prosegue, convertito, da dove era.
  Lo stesso accade al brano che segue da solo (vedi sotto).
- **Mantieni il DSD e silenzia le altre sorgenti.** Nulla interrompe il
  flusso DSD. Le altre sorgenti instradate al dispositivo restano mute
  finché il brano DSD non finisce, e nel frattempo il lettore mostra un
  badge **Altre mute**. Il brano successivo del lettore stesso parte
  quando il brano DSD finisce: un brano PCM dopo il silenzio DSD, un brano
  DSD dello stesso tipo e della stessa frequenza DSD subito, continuando il
  flusso.

**Un album non resta DSD con l'impostazione predefinita.** Con *Prosegui il
brano DSD in PCM*, esce come DSD solo un brano DSD che parte su un
dispositivo inattivo. I brani che il lettore avvia da solo dopo (alla fine
di un brano) partono da un
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

Il badge **BP** nell'intestazione del lettore esiste solo su un lettore il
cui dispositivo Main funziona bit-perfect. Si accende (bordo e testo verdi;
grigio attenuato quando è spento) finché il brano attuale arriva inalterato al
suo dispositivo Main. Tutte queste condizioni devono valere:

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
- il brano suona a guadagno unitario: un'uscita bit-perfect non viene mai
  normalizzata, ma un brano partito con un guadagno di normalizzazione prima
  che il suo dispositivo diventasse bit-perfect mantiene quel guadagno finché
  non si ferma, e BP resta spento per lui;
- il volume è mantenuto al 100 % su un'uscita bit-perfect, non è in corso
  alcuna dissolvenza (Stop in dissolvenza, le rampe di pausa e ripresa e il
  breve stop senza clic di un brano sono le uniche dissolvenze rimaste su un
  lettore del genere) e nient'altro suona sulle stesse uscite.

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
2. Riproduci un file di prova lossless con nient'altro in riproduzione.
3. Registralo.
4. Confronta la registrazione con il file. Per esempio, con SoX, inverti
   uno dei due e mixali: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`
   dopo aver allineato i loro inizi. Ogni campione della differenza deve
   essere zero.
5. Verifica un cambio di frequenza: in modalità Continua, con il **Mix
   automatico al punto MIX** attivo, riproduci un brano a 44,1 kHz che ha un
   punto MIX seguito da uno a 96 kHz. Il primo suona fino al suo cue-out senza
   dissolvenza; dopo un breve silenzio il convertitore mostra 96 kHz sul
   secondo. Il badge BP si accende su entrambi.
6. Poi riproduci due brani alla stessa frequenza: il secondo segue il primo
   senza pausa e senza sovrapposizione.

I test automatici del progetto verificano la stessa proprietà all'interno
dell'applicazione, su un dispositivo simulato.

### DSD su un convertitore reale {#dsd-on-a-real-converter}

I test automatici verificano il DSD solo su dispositivi simulati. Il DoP e il
DSD nativo non sono stati provati dal progetto su un convertitore reale. Per
verificarne uno:
1. Imposta il dispositivo su **DoP** (o su **DSD nativo** su Linux) e
   riproduci un file DSD con nient'altro in riproduzione. L'intestazione deve mostrare **DSD**, e il display del convertitore
   dovrebbe mostrare la frequenza DSD (per esempio DSD64) invece di una
   frequenza PCM. Un convertitore che mostra una frequenza PCM o riproduce
   rumore non riconosce il flusso: torna a **Converti in PCM**.
2. Ascolta se c'è un clic o una raffica di rumore all'inizio, allo Stop, alla
   fine del brano. Un clic significa che il
   convertitore ha bisogno di un **Silenzio DSD** più lungo (Impostazioni →
   Uscite audio, Avanzate).
3. Avvia un cart o un altro lettore sullo stesso dispositivo, una volta con
   ciascuna impostazione di mixaggio, e verifica il comportamento descritto
   sopra.
4. Su Linux, per verificare il DSD nativo senza l'applicazione, esegui
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test it -- --ignored conformance::native_dsd_on_a_real_device`.
   Apre il dispositivo in DSD nativo a DSD64 e riproduce un secondo di
   silenzio DSD. Deve passare, e il convertitore dovrebbe agganciarsi a
   DSD64.
5. Con file DSD in `test-music/`, `cargo test --release -p fp-engine --test it dsd_real_music:: -- --ignored`
   li riproduce attraverso il motore su un dispositivo simulato e confronta
   le parole con i byte del file (vedi [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
