# Lettori

Ogni colonna è un lettore. I lettori sono indipendenti: ciascuno ha le
proprie schede di playlist, il proprio trasporto, il proprio volume e le
proprie uscite.

![Lettore 1 in onda: intestazione, copertina, titolo, brano successivo, trasporto, conto alla rovescia, meter, fader e forma d'onda](../../images/guide/player.png)

## Intestazione {#header}

| Elemento | Significato |
|---|---|
| `P1` … `Pn` | Numero del lettore (il tasto numerico che lo fa suonare) |
| Punto di stato ed etichetta | **In onda** (rosso), **In pausa** (ambra), **Fermo** (grigio) |
| Badge **Mix** / **Dissolvenza** | È in corso una dissolvenza incrociata verso il brano successivo, o uno stop in dissolvenza |
| Badge **Stop alla fine** | Il lettore si ferma quando finisce il brano attuale |
| Badge **Ripeti** / **Stop dopo** | Il brano attuale si ripete, o ferma il lettore quando finisce, per un suo segno nel menu della playlist. Passa il puntatore per leggere la frase completa. Il pulsante **Stop alla fine** del lettore ha la precedenza: finché è attivo, si vede solo il suo badge |
| **BP** / **DSD** | **BP** è acceso finché il brano attuale arriva inalterato al suo dispositivo Main. **DSD** lo sostituisce mentre un brano DSD esce come DSD, inalterato (vedi [Uscita bit-perfect](bit-perfect.md)). **Altre mute** compare accanto quando quel flusso DSD tiene le altre sorgenti fuori dall'uscita |
| **SINGLE** \| **CONT** | Modalità di riproduzione (vedi sotto): un unico controllo unito, la metà accesa è la modalità attiva |
| **CUE** | Preascolta il brano successivo sull'uscita CUE |

## Riga delle informazioni {#info-row}

- **Copertina** del brano in onda, o un segnaposto a forma di vinile.
- **Titolo e artista** del brano in onda. Un lettore fermo mostra il brano
  che Play avvierà (il suo successivo), con la sua copertina, la sua durata e
  la sua forma d'onda, pronto al cue-in, o dove hai fatto clic sulla sua
  forma d'onda.
- Un brano riprodotto o caricato prima che la sua analisi sia finita ha già
  la sua durata quando la fornisce l'intestazione del file: il conto alla
  rovescia, `trascorso / totale` e il clic per spostarsi funzionano subito,
  su una linea piatta finché la forma d'onda non è pronta. Quando
  l'intestazione non la memorizza (file AAC grezzi, file MP3 senza frame di
  durata, Matroska e WebM), il totale mostra «—» e non si può fare clic sulla
  forma d'onda finché l'analisi non termina.
- **Meter stereo** (nella colonna a destra del lettore, accanto al fader;
  occupa la riga delle informazioni e il trasporto): il livello che il
  lettore emette, dopo il suo volume.
  - Una barra continua per canale, sulla scala dello standard del tipo di
    meter (per impostazione predefinita il meter di picco digitale: da −60 a
    0 dBFS, con i 20 dB superiori che occupano metà dell'altezza). La scala è
    un righello su ciascun lato delle barre, con le etichette nelle unità
    proprie del meter su entrambi i lati. Il suo estremo superiore e quello
    inferiore (per il meter digitale, il fondo scala) sono sempre segnati;
    ogni etichetta ha una tacca su ciascun righello, e le tacche più corte
    tra le etichette funzionano come quelle di un righello di misura:
    equidistanti su valori tondi, ogni 1 dB sul meter EBU e ogni 5 dB sotto
    −20 sul digitale quando il meter è abbastanza alto, e ogni 2, 2,5, 5 o
    10 dB (o nessuna) dove è troppo basso per averle. Sul meter digitale (e
    su quello personalizzato), un meter alto etichetta più valori: ogni 1 dB
    da −20 a 0, e ogni 5 dB tra i segni dei 10 dB sotto −20 (−45, −55),
    sempre equidistanti e solo dove entrano senza che le etichette si
    tocchino; gli altri tipi di meter mantengono le etichette previste dal
    loro standard. Il livello di allineamento (−18 dBFS sul meter digitale) è
    una tacca bianca più spessa su entrambi i righelli. Non viene disegnato
    nulla sopra le barre né tra di esse, quindi ciò che vedi nelle barre è
    solo il livello, la tenuta del picco e i colori.
  - La barra è verde, gialla dal livello di avviso (−9 dBFS) e rossa dal
    livello di pericolo (−3 dBFS). Gli altri tipi di meter diventano rossi
    dove lo fa la loro scala (da 0 VU, dal massimo consentito su un PPM).
  - I meter K-System mostrano due sezioni: la barra piena è il livello
    medio (RMS) e la parte più tenue sopra di essa arriva al picco. I loro
    colori sono quelli del K-System: verde sotto 0, ambra da 0 a +4, rosso
    sopra.
  - I meter di picco digitale, K-System e personalizzati mantengono per un
    istante il livello più alto acceso come una linea (la tenuta del picco;
    la sua durata è **Tenuta del picco** in
    [Impostazioni → Meter](settings.md#meters), e 0 la disattiva). I meter
    PPM EBU, PPM DIN e VU non hanno tenuta.
  - Il numero sopra è il livello più alto da quando è iniziata la voce, in
    dBFS, rosso nella zona di pericolo. Rimane dopo uno stop e riparte quando
    suona una voce (la successiva o la stessa di nuovo), o quando ci fai
    clic.
  - Il numero sotto è la loudness in LUFS (EBU R128), verde entro ±1 LU dal
    riferimento (−23 LUFS).
  - Il tipo di meter e ogni livello si possono cambiare in
    [Impostazioni → Meter](settings.md#meters).
  - **Letture sopra 0 dBFS.** Il meter mostra ciò che il lettore emette, e
    può superare il fondo scala. La barra si ferma in cima alla scala, quindi
    lo stesso rosso indica 0 dBFS e qualsiasi valore superiore; solo il
    numero sopra dice di quanto, con il suo segno (per esempio `+3.5`).
    - Un file può già contenere livelli sopra il fondo scala (un file float,
      o un file con perdita i cui picchi decodificati lo superano).
    - La conversione della frequenza può creare picchi tra i campioni: un
      segnale che tocca 0 dBFS legge circa +3 dBFS dopo 44,1 → 48 kHz.
      L'opzione di true peak legge anche questi picchi.
    - Il meter legge ogni lettore da solo, non la somma sul dispositivo: due
      lettori su una stessa uscita possono sommarsi oltre il fondo scala
      senza che nessuno dei due meter lo mostri.
    - Nel lettore nulla aggiunge guadagno oltre il 100 %. Un dispositivo a
      numeri interi taglia al fondo scala; un dispositivo float riceve il
      livello così com'è e il sistema audio o il driver lo taglia.
- **Fader del volume** (a destra del meter, alto quanto lui): trascinalo o
  usa la rotella del mouse, un passo per scatto. Il tooltip mostra il livello
  in dB; la cima è 0 dB e il fondo è il silenzio.
- **Titolo, artista** e la riga del **successivo**, con un quadrato verde.
  Quando il CUE è attivo, la posizione di preascolto è mostrata in blu.

## Trasporto {#transport}

| Pulsante | Azione |
|---|---|
| **Play / Successivo** (grande) | Fermo: avvia il brano successivo. In onda: passa in dissolvenza al brano successivo (il tempo di dissolvenza si imposta in [Impostazioni](settings.md)). In pausa: riprende. Con il brano in onda impostato come successivo, Play riavvia quel brano con la consueta dissolvenza. |
| **Stop** | Si ferma subito (con una breve rampa anti-clic) |
| **Stop in dissolvenza** | Dissolve fino al silenzio e si ferma |
| **Pausa** | Mette in pausa o riprende; lampeggia in ambra mentre è in pausa |
| **Stop alla fine** (un triangolo di play e poi un quadrato) | Si ferma quando finisce il brano attuale, una volta. In modalità SINGLE è disponibile solo mentre il brano attuale si ripete: termina la ripetizione quando finisce il passaggio in corso. Per fermarsi dopo un brano ogni volta che suona, o per ripetere un brano, usa il suo menu nella playlist (vedi [Playlist](playlists.md)) |
| **Precedente** (una barra e due triangoli) | In onda: torna in dissolvenza al brano che questo lettore ha suonato prima, come fa Successivo. Premi di nuovo per continuare a tornare indietro. Il brano che hai lasciato diventa il successivo. |
| **Riavvia** (una barra e un triangolo) | Torna all'inizio del brano attuale (il suo cue-in). Un lettore in pausa resta in pausa. |

I pulsanti che al momento non possono agire sono attenuati: Stop e Riavvia
senza nulla di caricato, Pausa e Stop in dissolvenza da fermo, Precedente
senza un brano precedente o durante una dissolvenza. Un lettore ricorda gli
ultimi 50 brani che ha suonato (`players.history_len` in `config.json`, da 0
a 1000).

## Modalità {#modes}

- **CONT (continua):** al punto MIX il lettore avvia il brano successivo e
  sovrappone la fine di quello attuale. Vedi
  [Marker e mix](markers-and-mixing.md).
- **SINGLE:** ogni brano si ferma alla sua fine. *Stop alla fine* non è
  disponibile in questa modalità, perché ogni brano si ferma già, tranne
  mentre il brano attuale si ripete: allora termina la ripetizione quando
  finisce il passaggio in corso.

## Conto alla rovescia {#countdown}

Il numero grande è il tempo che manca alla fine del brano (il suo cue-out),
con i decimi. Il tempo trascorso e il totale sono nella riga sotto la forma
d'onda, a destra. Negli ultimi secondi prima della fine (10 per impostazione
predefinita, regolabili nelle Impostazioni) il conto alla rovescia lampeggia
in rosso.

## Forma d'onda {#waveform}

- La parte già suonata è disegnata con il colore della forma d'onda; il
  resto è più tenue.
- Il contorno mostra i picchi, in modo discreto; il corpo pieno al suo
  interno è il livello medio (RMS). In un brano forte i picchi riempiono
  l'altezza, e il corpo mostra comunque dove il brano è più piano o più
  forte.
- Un'area blu ombreggiata all'inizio segna l'**intro**, e un badge ne fa il
  conto alla rovescia. L'intro compare solo quando è stata impostata.
- Un'area arancione ombreggiata alla fine segna l'**outro**, con il proprio
  conto alla rovescia.
- Una linea ambra tratteggiata con l'etichetta **MIX** segna dove inizia il
  brano successivo in modalità continua. È attenuata in modalità single.
- Viene disegnato l'intero file. L'inizio e la fine silenziosi che la
  riproduzione salta (prima del cue-in e dopo il cue-out) sono disegnati più
  scuri, con una linea sottile dove la riproduzione inizia e finisce. Con
  **Usa cue-in e cue-out** disattivato (Impostazioni → Lettori) nulla è più
  scuro e le due linee sono attenuate: la riproduzione va dall'inizio alla
  fine del file, e il cue-in e il cue-out in questa guida indicano quei due
  estremi.
- Passa il puntatore per vedere il tempo sotto di esso. **Fai clic per
  spostarti** lì. Su un lettore fermo, un clic sceglie da dove **Play**
  avvia il brano successivo: la testina e il conto alla rovescia si
  spostano lì, e non suona nulla finché non premi Play. Scegliere un altro
  brano successivo, spostarlo o rimuoverlo, o Stop riportano al cue-in; lo
  fa anche qualsiasi altro modo di avviare un brano, e Riavvia, Precedente e
  l'avanzamento automatico usano sempre il cue-in. Un clic prima del cue-in
  (nell'inizio più scuro) sceglie il cue-in. Un clic al cue-out o dopo (nella
  coda più scura) annulla una scelta precedente: Play parte dal cue-in. Un
  clic è una pressione e un rilascio senza spostare il puntatore di più di
  pochi pixel.
- **Premere e trascinare** sposta lungo il brano la vista ingrandita, come
  afferrandola. Un trascinamento non salta mai, e senza zoom non fa nulla.
  Alt+trascinamento modifica ancora i marker.
- **Rotella del mouse** sulla forma d'onda: ingrandisce e rimpicciolisce
  attorno al puntatore, fino al massimo dettaglio dell'analisi.
  **Shift+rotella** (o una rotella laterale) si sposta lungo il brano.
  Mentre è ingrandita, la vista segue la posizione di riproduzione, tranne
  per 10 secondi dopo che l'hai ingrandita o spostata
  (`ui.follow_current_grace_secs`; 0 disattiva il seguito). **Vista
  completa**, nell'angolo in alto a destra, il rimpicciolimento fino in
  fondo o un nuovo brano mostrano di nuovo tutto il brano.

## CUE (preascolto) {#cue-pre-listen}

Premere **CUE** (o **Preascolta nel CUE** nel menu di un brano) riproduce il
brano sull'uscita CUE del lettore, per esempio le cuffie, senza toccare
l'uscita in onda, e apre una piccola **finestra CUE** per quel lettore.
Possono essere aperte più finestre, una per lettore. Vedi
[Impostazioni](settings.md) per scegliere il dispositivo CUE. Un lettore ha
bisogno di un'uscita Cue diversa dalla sua uscita Main: senza, **CUE** e
**Preascolta nel CUE** sono attenuati, e passandoci sopra il puntatore lo
dicono.

![La finestra CUE del lettore 4, che preascolta il suo brano successivo](../../images/guide/cue-window.png)

La finestra mostra:

- il titolo e l'artista;
- la forma d'onda dell'intero file con la posizione del CUE. Funziona come
  quella del lettore: fai clic per spostarti lì, ingrandisci con la
  rotella, trascina per spostarti lungo il brano, **Vista completa**, i
  conti alla rovescia di intro e outro, e i marker di intro, outro e MIX,
  che qui modifichi come sul lettore (vedi
  [Marker e mix](markers-and-mixing.md)). Un CUE riproduce l'intero file,
  quindi nulla è disegnato più scuro, il cue-in e il cue-out sono linee
  attenuate, e l'outro conta alla rovescia fino alla fine del file. Il suo
  zoom è indipendente: la forma d'onda del lettore non si muove. Mentre il
  CUE suona, una vista ingrandita ne segue la posizione come quella del
  lettore; un CUE in pausa mantiene la vista che hai impostato, così puoi
  ingrandire e posizionare i marker. Un CUE avviato dopo la chiusura della
  sua finestra, o su un altro brano, mostra l'intero file;
- il tempo trascorso e il tempo rimanente fino alla fine del file (un CUE
  riproduce file interi);
- **Pausa** / **Riprendi**, **Stop** e **Imposta come successivo**.
  **Imposta come successivo** rende il brano in CUE il successivo del
  lettore e lascia il CUE in riproduzione. È attenuato quando il brano è già
  il successivo. Se il brano in CUE è quello in onda, suonerà ancora una
  volta quando finisce il passaggio in corso.

Mentre il CUE è in pausa, il suo pulsante **Pausa** (mostrato come
**Riprendi**) lampeggia in ambra, come quello del lettore.

Uno spostamento su un CUE in pausa lo mantiene in pausa. Il pulsante di
chiusura della finestra, o **Stop**, ferma il CUE.

Mentre un CUE è in corso, impostare un successivo (doppio clic) o un singolo
clic su una riga sposta il CUE su quel brano, dal suo cue-in; se era in
pausa, riprende a suonare. Un brano il cui file manca o è illeggibile lascia
il CUE dov'è.
