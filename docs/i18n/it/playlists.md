# Playlist

## Schede {#tabs}

Ogni lettore ha una riga di schede, una per playlist. Tutti i lettori vedono
le stesse playlist; ciascun lettore sceglie quale mostrare. Un punto su una
scheda indica dove si trovano i brani del lettore: **rosso** per il brano in
onda, **verde** per il successivo.

Le schede si dividono la larghezza del lettore. Un nome che non entra
termina con «…»; passa il puntatore sulla scheda per leggerlo per intero.
Con molte playlist le schede smettono di restringersi a una larghezza
minima, agli estremi della riga compaiono delle frecce e la rotella del
mouse sulle schede le fa scorrere. La scheda che scegli, e quella che mostra
un lettore, viene portata in vista.

Cambiare scheda non modifica mai ciò che è in onda né ciò che è successivo.
Quando un brano finisce, il lettore prosegue nella playlist che contiene
quel brano.

Le playlist si creano, si rinominano e si eliminano in
[Impostazioni](settings.md). L'ultima playlist, e una playlist con un brano
in onda, non si possono eliminare.

## La tabella dei brani {#the-track-table}

![Una playlist: brani già suonati attenuati, il brano in onda in rosso, il brano successivo in verde, e il piè di pagina con il tempo rimanente](../../images/guide/playlist.png)

| Colonna | Contenuto |
|---|---|
| `#` | Posizione, con zeri iniziali; per il brano attuale e per il successivo la sostituisce un'icona |
| Titolo | Dai tag, o dal nome del file (`Artista - Titolo.mp3` viene diviso). Le icone di ripetizione e di stop dopo di un brano stanno prima del titolo |
| Artista | Dai tag; «Artista sconosciuto» quando non c'è |
| Album | Dai tag |
| Data | La data di registrazione così come la memorizza il file (`2019`, `2019-05` o `2019-05-14`, con un orario se c'è) |
| Genere | Dai tag |
| Durata | Durata di riproduzione, dal cue-in al cue-out (l'intero file con **Usa cue-in e cue-out** disattivato) |
| Intro | Quanto dura l'intro, da dove il brano inizia a suonare fino al suo marker di intro; vuota quando il brano non ha un marker di intro |
| Nome del file | Il nome del file, con la sua estensione |

Una nuova installazione mostra `#`, Titolo, Artista e Durata. Le altre
colonne sono facoltative; vedi **Scegliere le colonne** più sotto. Un brano
che non ha un valore mostra una cella vuota, tranne Artista, che mostra
«Artista sconosciuto».

Le colonne riempiono la tabella e mantengono le loro proporzioni quando la
finestra viene ridimensionata; le colonne di testo hanno più spazio.
Trascina i separatori delle intestazioni per cambiare le proporzioni: le
colonne a destra del separatore seguono il puntatore a ogni frame (si
dividono ciò che resta in proporzione alla loro larghezza), quelle alla sua
sinistra restano ferme, e le larghezze vengono salvate quando rilasci. Nessuna
colonna diventa più stretta del suo minimo. Le larghezze sono ricordate per
lettore; una colonna che mostri in seguito parte con la sua larghezza
predefinita e le altre mantengono le loro proporzioni.

### Scegliere le colonne {#choosing-the-columns}

Titolo e Durata sono sempre visibili. Ogni altra colonna si può mostrare o
nascondere, e qualsiasi colonna, anche queste due, si può spostare. L'elenco
è lo stesso per ogni lettore e ogni playlist, e viene salvato in
`config.json` come `ui.table_columns`. Tre modi per cambiarlo:

- **Impostazioni → Playlist → Colonne della tabella:** spunta le colonne da
  mostrare; le frecce spostano su o giù una colonna visibile (si leggono da
  sinistra a destra nelle tabelle). **Colonne predefinite** torna a `#`,
  Titolo, Artista e Durata.
- **Clic destro su un'intestazione:** un menu con una casella per ogni
  colonna facoltativa. Una colonna che mostri compare all'estremità destra;
  trascinala da lì.
- **Trascina un'intestazione** su un'altra: rilasciala sulla metà sinistra
  di un'intestazione per mettere la colonna prima di essa, sulla metà destra
  per metterla dopo. Rilasciare altrove non fa nulla.

Un nome in `ui.table_columns` che questa versione non conosce viene
ignorato, e un Titolo o una Durata mancante viene riaggiunto.

All'apertura dell'applicazione, ogni tabella scorre in modo che il brano
successivo del suo lettore sia al centro della tabella (per quanto lo
permettano gli estremi dell'elenco). Succede una sola volta, all'avvio, e
solo quando il brano successivo è nella playlist che la tabella mostra.

Quando un lettore passa a un altro brano, la sua tabella mostra la playlist
di quel brano e fa scorrere la sua riga in cima, a meno che tu non abbia
usato la tabella negli ultimi 10 secondi (l'hai fatta scorrere, hai
trascinato un brano, hai aperto il menu di un brano o hai fatto clic su una
scheda): allora aspetta finché non la lasci stare per tutto quel tempo. Il
tempo è `ui.follow_current_grace_secs` in `config.json`; 0 disattiva il
seguito.

I lettori sono indipendenti: più lettori possono mostrare la stessa
playlist, ciascuno con il proprio brano successivo, i propri segni di
riprodotto e i propri tempi nel piè di pagina. Riprodurre, fermare o saltare
su un lettore non sposta mai il successivo di un altro lettore. Lo stesso
brano può anche essere in onda su due lettori contemporaneamente.
Modificare la playlist (aggiungere, spostare o rimuovere voci) o un file che
diventa illeggibile può comunque cambiare il successivo di qualsiasi lettore
che la mostri.

Colori delle righe:

| Riga | Significato |
|---|---|
| **Rossa**, con un'icona di altoparlante (o di pausa) | In onda su questo lettore. Può mostrare anche la freccia verde: il brano in onda è anche il successivo, quindi suona ancora una volta |
| **P2** rosso (o un altro numero) nella colonna del numero | In onda su quel lettore |
| **Verde**, con una freccia | Il brano successivo di questo lettore |
| Attenuata | Già suonato su questo lettore |
| File con una croce / icona di avviso | File mancante / illeggibile (viene saltato); passa il puntatore sulla riga: il popup inizia con il motivo, in ambra, poi i consueti campi. Un file mancante viene cercato di nuovo ogni 30 s (`tuning.missing_recheck_ms`). |
| Frecce di ricarica a destra del titolo | Analizzato da una versione precedente; suona comunque con quell'analisi. **Impostazioni → Analisi → Analizza i brani obsoleti** lo aggiorna (i brani su un lettore vengono comunque aggiornati) |
| Clessidra a destra del titolo | Il brano è in attesa della sua analisi (passa il puntatore sulla clessidra: *Analisi in attesa*). Suona comunque, e la clessidra scompare quando l'analisi finisce |
| Viola | Selezionata |

**Tooltip del brano.** Passa il puntatore su una riga per un istante per vedere titolo, artista, album, data, genere, durata, formato (tipo, frequenza di campionamento e profondità di bit, quando noti) e il percorso del suo file. Un campo che il file non ha viene omesso. Il popup è l'unica informazione al passaggio del puntatore su una riga, e non si sposta mai mentre è mostrato. Per un file mancante o illeggibile inizia con il motivo.

## Mouse {#mouse}

- Il **clic** seleziona un brano. Il **doppio clic** lo rende il brano
  successivo di questo lettore. Sul brano in onda lo fa suonare ancora una
  volta quando finisce il passaggio in corso.
- Il **clic destro** apre il menu contestuale:

![Il menu contestuale di un brano](../../images/guide/track-menu.png)

| Voce | Azione |
|---|---|
| Riproduci ora | Avvia subito questo brano (con un mix se il lettore è in onda) |
| Imposta come successivo | Come il doppio clic. Sul brano in onda suona ancora una volta, dall'inizio, quando finisce il passaggio in corso (con un mix come Ripeti, senza pause), poi il lettore prosegue. Agisce una volta sola. Stop alla fine, la modalità SINGLE e un segno Stop dopo fermano comunque prima il lettore. Mentre un CUE è in corso, si sposta sul nuovo successivo |
| Preascolta nel CUE | Lo riproduce sull'uscita CUE (apre la finestra CUE). Attenuata quando il lettore non ha un'uscita Cue distinta dalla sua Main |
| Modifica i tag… | Apre l'editor dei tag per questo brano. **Salva** scrive le modifiche nel file audio; **Annulla** (o Esc, quando non è in corso un salvataggio) chiude senza scrivere. La voce è attenuata, con il motivo al passaggio del puntatore, mentre il brano è in onda, nel CUE o su un cart in riproduzione, finché i suoi tag non sono stati letti, quando il file manca e per i formati i cui tag non si possono scrivere (per esempio il DSD) |
| Rianalizza | Analizza di nuovo ora questo brano, qualunque sia il suo stato. Anche un file corretto che era illeggibile viene recuperato da solo (vedi [Risoluzione dei problemi](troubleshooting.md)). I marker manuali vengono mantenuti |
| Aggiungi brani sotto… | Scegli i file da inserire dopo questo brano |
| Duplica | Inserisce sotto una copia non ancora suonata (con i suoi segni di ripetizione e di stop dopo) |
| Ripeti questo brano | Spuntala per riprodurlo di continuo, senza pause, finché non premi Play (successivo), Precedente, Stop o Stop in dissolvenza, o attivi Stop alla fine. Pausa lo lascia in ripetizione. Un'icona di ripetizione compare prima del titolo |
| Stop dopo questo brano | Spuntala per fermare il lettore quando questo brano finisce, ogni volta che suona (in qualsiasi modalità). A differenza del pulsante **Stop alla fine** del lettore, il segno resta con il brano e viene salvato con la playlist. L'icona di stop dopo compare prima del titolo. Ha la precedenza su Ripeti |
| Sposta in ▸ | Lo sposta in fondo a un'altra playlist |
| Rimuovi dalla playlist | Lo rimuove; non è possibile mentre è in onda |

## Modificare i tag {#editing-tags}

**Modifica i tag…** apre una finestra per un solo brano. Finché è aperta
nessuna scorciatoia da tastiera agisce, e i file trascinati sulla finestra
dell'applicazione vengono ignorati.

![L'editor dei tag di un file FLAC, con copertina, titolo, artista, album, data e genere](../../images/guide/tag-editor.png)

- **Cosa vedi.** L'editor legge il file quando si apre (nel frattempo mostra
  «Lettura dei tag…»). Sempre visibili: titolo, artista, album, artista
  dell'album, data, numero della traccia e totale, numero del disco e
  totale, genere, compositore e commento. Visibili quando il file li ha:
  sottotitolo, raggruppamento, BPM, tonalità iniziale, atmosfera, ISRC,
  editore, numero di catalogo, copyright, artista originale, album originale,
  data di uscita originale, paroliere, direttore, remixer, arrangiatore,
  interprete, lingua, codificato da, testo, ordinamento titolo, ordinamento
  artista, ordinamento album, ordinamento artista dell'album, ordinamento
  compositore e sito web dell'artista.
- **Aggiungi campo.** Il menu sotto i campi elenca gli altri campi. Offre
  solo ciò che il formato di tag del file può memorizzare (un WAV con RIFF
  INFO, un AIFF o un vecchio tag ID3v1 memorizzano meno campi di ID3v2, FLAC
  o MP4), ed è attenuato quando non resta nulla da aggiungere. Uno dei campi
  sempre visibili che il formato non può memorizzare è disattivato con una
  nota. Svuotare un campo lo rimuove dal file; un campo aggiunto e lasciato
  vuoto non viene scritto.
- **Più valori.** I campi che possono contenere più valori (artista, artista
  dell'album, genere, compositore, atmosfera e i crediti come paroliere,
  direttore, remixer, arrangiatore e interprete, e lingua) mostrano un valore
  per riga; **Salva** scrive un valore per riga nel modo proprio del
  formato. Commento e testo sono testo libero su più righe.
- **Controlli.** Data e data di uscita originale sono in ISO 8601 (`2019`,
  `2019-05` o `2019-05-14`, facoltativamente con un orario); numero della
  traccia e del disco, i loro totali e i BPM sono numeri interi, e un totale
  richiede il suo numero. Un campo con un valore non valido viene segnato e
  **Salva** resta disattivato. Un valore che il file aveva già e che non hai
  toccato viene mantenuto com'è.
- **Campi troppo lunghi.** Un campo il cui testo è più lungo di
  `limits.max_tag_chars`, o che contiene più valori di
  `limits.max_tag_values`, viene mostrato in sola lettura con la nota
  «Troppo lungo per modificarlo qui; viene mantenuto com'è nel file». Non
  viene mai riscritto, quindi un salvataggio non può troncarlo.
- **Cosa viene mantenuto.** Tutto ciò che l'editor non mostra (altre chiavi
  standard, chiavi personalizzate, immagini diverse dalla copertina
  frontale, frame binari) resta nel file con gli stessi valori. L'editor
  dice quanti tag di questo tipo vengono mantenuti (e «altri» quando il
  formato contiene frame che non si possono contare). Il salvataggio
  ricodifica le voci che l'editor mappa, quindi una voce mantenuta può
  differire nei byte (codifica del testo, ordine dei frame) ma non nel
  valore.
- **La copertina.** L'editor mostra la copertina frontale, o la prima
  immagine del file quando non c'è una copertina frontale, come miniatura.
  - **Cambia…** apre una finestra di dialogo per scegliere un'immagine JPEG
    o PNG (al massimo `limits.max_cover_bytes`, e deve poter essere
    decodificata). In caso contrario, l'editor dice perché e non cambia
    nulla.
  - **Rimuovi** cancella la copertina frontale. È disattivato quando il file
    non ha una copertina frontale: un'immagine mostrata solo perché non c'è
    una copertina frontale è solo per la visualizzazione e viene mantenuta
    com'è.
  - Una copertina presente nel file ma che non si può mostrare (un'immagine
    che non si decodifica, o una GIF, BMP o WebP) viene segnalata con
    «Questa copertina non si può mostrare; viene mantenuta com'è». **Cambia…**
    e **Rimuovi** funzionano comunque.
  - La modifica viene scritta da **Salva** e scartata da **Annulla**. Le
    copertine posteriori e ogni altra immagine non vengono mai toccate. Un
    formato senza posto per le immagini (WAV con RIFF INFO, AIFF, ID3v1)
    mostra l'area disattivata. Dopo un salvataggio, la copertina del lettore
    mostra la nuova copertina.
- **Come funziona un salvataggio.** Il file viene copiato accanto
  all'originale, la copia riceve i tag, viene sincronizzata e sostituisce
  l'originale, quindi un errore lascia il file com'era. Il motivo compare
  nell'editor, che resta aperto per riprovare, e nella barra di stato.
  Vengono scritti solo i campi che hai cambiato. Dopo un salvataggio la
  tabella mostra subito i nuovi tag, e i marker e la forma d'onda vengono
  mantenuti. Se il file non ha mantenuto un campo che hai cambiato, la barra
  di stato lo nomina.
- **Dopo un aggiornamento.** I brani di una versione precedente ricevono in
  silenzio, in background, data, genere e altri tag (senza un'analisi
  completa).

## Trascinamento {#drag-and-drop}

- Trascina un brano nell'elenco per riordinarlo. Una linea viola mostra dove
  atterrerà: si trova sul confine di riga più vicino al puntatore, e solo
  nell'elenco sotto il puntatore. Rilasciare sopra l'intestazione, il bordo
  di una colonna, la barra di scorrimento o una finestra che copre l'elenco
  (la finestra CUE) non rilascia nulla.
- Mentre trascini un brano, tieni il puntatore vicino al bordo superiore o
  inferiore di un elenco per farlo scorrere: più sei vicino al bordo, più
  va veloce, e si ferma agli estremi dell'elenco o quando ti allontani dal
  bordo. Anche la rotella del mouse fa scorrere l'elenco durante il
  trascinamento. La linea viola continua a seguire il puntatore mentre
  l'elenco si muove. Anche trascinare file dal file manager su un elenco lo
  fa scorrere allo stesso modo dove il sistema comunica la posizione del
  puntatore, una volta che sposti il puntatore sull'elenco.
- Trascinalo sull'elenco di un altro lettore per spostarlo lì.
- Trascinalo su una scheda per aggiungerlo in fondo a quella playlist.
- Rilascia file o cartelle dal file manager su un elenco per inserirli nella
  posizione di rilascio. Sopra l'intestazione, il bordo di una colonna, la
  barra di scorrimento o una finestra che copre l'elenco non viene inserito
  nulla. Se il sistema non comunica la posizione, vanno in fondo all'elenco
  mostrato.

## Piè di pagina {#footer}

**+ Aggiungi** apre una finestra di dialogo per scegliere i file, che parte
dalla cartella della musica impostata nelle Impostazioni. **Azzera** (l'icona
a freccia accanto) cancella il segno attenuato di «già suonato» di ogni
brano della playlist, per ogni lettore, dopo aver chiesto «Cancellare il
segno di riprodotto da ogni brano di questa playlist?» (**Annulla**, Esc o
un clic fuori mantengono i segni). Il brano in onda mantiene il suo stato e
viene segnato quando il lettore lo lascia. Il pulsante è attenuato quando
non c'è nulla da cancellare. Il piè di pagina mostra anche il numero di
brani, il tempo rimanente nella playlist e la sua durata totale.

## File di playlist {#playlist-files}

- **Importazione:** Impostazioni → Playlist → **Importa M3U / PLS…**, oppure
  trascina un file `.m3u`, `.m3u8` o `.pls` sulla finestra. Diventa una nuova
  playlist con il nome del file.
  - I percorsi relativi vengono risolti rispetto alla cartella del file di
    playlist.
  - Gli indirizzi `file://` vengono interpretati.
  - I file che non si trovano vengono comunque aggiunti, segnati come non
    disponibili.
  - Gli stream Internet vengono ignorati; un messaggio dice quanti.
- **Esportazione:** il pulsante **M3U** su ogni playlist nelle Impostazioni
  la salva come file M3U8 con titoli, durate e percorsi completi.
