# Libreria

La libreria è l'insieme di tutti i brani che Fauste Player conosce: ogni file
che hai aggiunto a una playlist, a un cart o alla libreria stessa è un
brano, con i suoi tag, i suoi marker e le sue impostazioni di loudness. La
finestra della libreria li elenca, li raggruppa in *categorie*, li trova per
testo e li invia alle playlist e alla cartwall. È una finestra sul desktop e
su un tablet; un telefono mostra invece la libreria nella sua scheda
**Libreria** (vedi [Su un telefono](#on-a-phone)).

## Aprire la finestra {#opening-the-window}

Fai clic sulla tessera **Libreria** nella barra superiore. È accesa mentre la
finestra è aperta; fai di nuovo clic su di essa, o sul pulsante di chiusura
della finestra, per chiuderla. La finestra fluttua sopra i lettori e la
cartwall e non li blocca: nulla al suo interno ferma l'audio o le scorciatoie
della finestra principale.

Spostala dalla sua barra del titolo e ridimensionala dai bordi e dagli
angoli. La barra del titolo ha tre pulsanti: **Riduci a icona la libreria**
(resta solo la barra del titolo, dove era la finestra, o in cima all'area
principale quando la finestra è ingrandita; fai di nuovo clic per espandere),
**Ingrandisci la libreria** (la finestra riempie l'area principale;
**Ripristina la libreria** la riporta com'era) e **Chiudi la libreria**. Una
finestra che non entrerebbe più, dopo che la finestra principale si rimpicciolisce o si sposta
su uno schermo più piccolo, viene riportata dentro l'area principale.

La posizione e la dimensione della finestra, lo stato ridotto a icona o
ingrandito, la larghezza dell'albero, le colonne, l'ordinamento e la
categoria selezionata vengono salvati con la sessione e tornano al prossimo
avvio. Viene salvato anche se la finestra è aperta, ma per questo non suona
nulla da solo. Il testo nel campo di ricerca non viene salvato.

## Categorie {#categories}

L'albero a sinistra raggruppa i brani. In cima, **Tutti i brani**; poi le tue
categorie, che possono contenere sottocategorie; in fondo, **Senza
categoria**, i brani che non appartengono ad alcuna categoria. Ogni voce
mostra il suo colore e un conteggio: una categoria conta i propri brani e
quelli delle sue sottocategorie. Un brano appartiene al massimo a una
categoria.

- **Fai clic** su una voce per elencarne i brani nella tabella. Una categoria
  elenca anche i brani delle sue sottocategorie.
- La **freccia** davanti a una categoria apre o chiude le sue
  sottocategorie. Quali sono aperte viene salvato.
- **Nuova categoria** (＋ sopra l'albero) chiede un nome, un colore e un
  *tipo*. Una nuova categoria prende il colore e il tipo dalla categoria in
  cui viene creata; il tipo è ciò che un brano prende quando entra nella
  categoria, e cambiarlo in seguito lascia i brani già presenti com'erano. Un
  nome non può essere vuoto e non può ripetere il nome di una categoria
  sorella; le categorie si annidano solo fino a un limite.
- **Fai clic destro** su una categoria (o tieni premuto su uno schermo
  tattile) per **Nuova sottocategoria…**, **Rinomina…**, **Colore**, **Tipo**
  ed **Elimina…**. Sul desktop il menu ha anche **Sorveglia cartella…**
  (**Cambia cartella sorvegliata…** e **Smetti di sorvegliare la cartella**
  una volta che ne sorveglia una) e **Rimuovi i brani mancanti…**; vedi
  [Cartelle sorvegliate](#watched-folders) e
  [Rimuovere i brani](#removing-tracks).
- **Trascina** una categoria per spostarla: su un'altra categoria per
  annidarla, o nello spazio tra due categorie per collocarla lì. Una
  categoria non può andare dentro se stessa. Su uno schermo tattile l'albero
  accetta solo tocchi e scorre sotto un dito; una categoria non può essere
  trascinata lì.

**Elimina…** nomina ciò che se ne va con la categoria: le sue sottocategorie
e ogni brano al loro interno vengono rimossi dalla libreria. I file non
vengono mai eliminati. Se uno qualsiasi di quei brani è in uso, una seconda
domanda dice quante voci di playlist e quanti cart li usano: **Rimuovi
comunque** rimuove le voci dalle loro playlist e svuota i cart. Nulla viene
rimosso finché uno di essi è in onda; un avviso dice cosa suona.

Lo stesso albero è in **Impostazioni → Libreria**, per creare, rinominare,
cambiare il colore e il tipo, spostare ed eliminare le categorie senza aprire
la finestra (non ha *Tutti i brani*, *Senza categoria* né selezione).

## Cartelle sorvegliate {#watched-folders}

Sul desktop una categoria può sorvegliare una cartella. Fai clic destro su di
essa e scegli **Sorveglia cartella…**, poi scegli la cartella. La categoria
mostra un segno ⟳ prima del suo conteggio; passaci sopra il puntatore per
leggere la cartella.

- Ogni file audio nella cartella e nelle sue sottocartelle entra nella
  categoria. Le sottocartelle non creano sottocategorie.
- Un brano che è già in un'altra categoria resta lì. Un brano che non è in
  alcuna categoria entra in questa. Impostare, cambiare o smettere di
  sorvegliare non sposta mai un brano: i brani di una cartella che non è più
  sorvegliata restano nella loro categoria.
- Mentre l'applicazione è in esecuzione, la cartella viene seguita: un nuovo
  file entra dopo che è stato fermo per un momento
  (`library.watch_debounce_ms`, 1 secondo per impostazione predefinita, così
  un file ancora in copia non viene letto a metà). La cartella viene anche
  analizzata a ogni avvio dell'applicazione, per recuperare ciò che è cambiato
  mentre era chiusa.
- Un file che sparisce dalla cartella (eliminato, spostato o rinominato)
  lascia il suo brano nella libreria, segnato come mancante, con i suoi
  marker. Se il file ritorna, il brano torna normale da solo. Un file
  rinominato entra come un nuovo brano con il suo nuovo nome.
- Le cartelle sorvegliate possono essere annidate. Un file sotto due di esse
  va alla categoria di quella più interna; smettere di sorvegliarne una lascia
  l'altra seguita. La stessa cartella non può essere sorvegliata da due
  categorie: l'applicazione dice quale categoria ce l'ha già.
- Nulla di ciò che fa una cartella sorvegliata carica o riproduce un brano.

Se la cartella non può essere seguita, il segno ⟳ cambia colore e il suo
tooltip ne dice il motivo: *Cartella sorvegliata non trovata: …. Viene cercata
di nuovo.* (per esempio un'unità non montata), *Impossibile leggere la
cartella sorvegliata: …. Si riprova.*, oppure *Le modifiche in … non vengono
seguite; la cartella è analizzata all'avvio e si riprova.* (il sistema ha
rifiutato di sorvegliarla). Una cartella memorizzata come percorso non
completo (modificato a mano) non viene sorvegliata affatto, e il segno lo
dice.

Ogni 30 secondi (`tuning.missing_recheck_ms`) l'applicazione controlla che
ogni cartella sorvegliata sia ancora lì, così viene segnalata anche un'unità
scollegata mentre è in esecuzione. Una cartella in difficoltà viene
riprovata a quel ritmo e analizzata appena ritorna. Una cartella che non c'è,
non si può leggere, o è vuota mentre la libreria ha ancora brani provenienti
da essa (un'unità o una condivisione non montata spesso sembra una cartella
vuota) conta come non trovata. La sorveglianza allora non segna come mancante
nessuno dei suoi brani e non rimuove nulla, anche quando la cartella sta
dentro un'altra cartella sorvegliata.

Se svuoti di proposito una cartella sorvegliata, l'avviso resta finché non
ci sono di nuovo file o smetti di sorvegliarla, e i suoi brani restano nella
libreria senza essere segnati come mancanti. Per eliminarli, selezionali
nella tabella e scegli **Rimuovi dalla libreria…**, oppure scegli
**Rianalizza** su di essi: l'analisi segna come mancanti i brani il cui file è
sparito, e **Rimuovi i brani mancanti…** poi li rimuove (se la cartella è
ancora sorvegliata, il suo avviso scompare al controllo successivo).

Un'eccezione alla regola qui sopra: un brano che non è mai stato analizzato
viene comunque segnato come mancante quando l'analisi non trova il suo file,
ovunque sia.

Una sottocartella che non si può leggere viene saltata e il log lo dice; i
collegamenti a cartelle (collegamenti simbolici a cartelle) non vengono
seguiti.

I nuovi file vanno all'analisi dopo tutto ciò di cui un lettore o un cart ha
bisogno, quindi una cartella con migliaia di file richiede un po' di tempo;
l'avanzamento dell'analisi lo mostra.

Lo stesso menu e lo stesso segno sono in **Impostazioni → Libreria**. Non
sono su Android, che non sorveglia le cartelle: lì una categoria può prendere
i suoi file da una cartella del dispositivo che rileggi quando scegli (vedi
[Android → Libreria](android.md#library)).

## La tabella dei brani {#the-track-table}

La tabella elenca i brani della voce selezionata, nelle stesse colonne delle
tabelle delle playlist (quelle scelte in **Impostazioni → Playlist → Colonne
della tabella**, senza la posizione `#`) seguite da **Categoria** e **Tipo**.
Una cella stretta a sinistra segnala un brano il cui file manca o non si può
leggere. Un brano in attesa della sua analisi è segnalato a destra della sua
cella **Titolo** (quindi solo quando la colonna Titolo è mostrata).

- Trascina il separatore di un'intestazione per cambiare la larghezza di una
  colonna; fai clic su un'intestazione per ordinare per essa e di nuovo per
  invertire. Le larghezze e l'ordinamento vengono salvati.
- Il **clic** seleziona una riga, **Ctrl** (**Cmd** su macOS) **clic**
  aggiunge o toglie una riga, **Shift clic** seleziona le righe tra l'ultimo
  clic e questo. La selezione mantiene solo le righe elencate: una riga che
  lascia la tabella, per una ricerca, un'altra categoria o una rimozione,
  lascia la selezione. Ogni azione si applica alle righe selezionate
  nell'ordine della tabella.
- **Aggiungi file** (sopra la tabella) sceglie dei file audio e li aggiunge
  alla libreria, nella categoria selezionata (senza categoria quando è
  selezionato *Tutti i brani* o *Senza categoria*).

Su uno schermo tattile le righe accettano solo tocchi e scorrono sotto un
dito; una pressione prolungata apre il menu, e i brani si inviano con i menu
(sotto), non trascinando.

## Ricerca {#search}

Il campo sopra la tabella trova i brani per testo. Cerca nel titolo,
nell'artista, nell'album, nell'artista dell'album, nel compositore, nel
genere, nel commento e nel nome del file, ignorando maiuscole e accenti, e un
brano viene elencato quando contiene tutte le parole che hai digitato (in
qualsiasi ordine e in qualsiasi di quei campi). Cerca nella voce selezionata;
attiva **Tutta la libreria** per cercare in ogni brano qualunque sia la
selezione. L'ambito viene salvato; il testo no. Svuota il campo per vedere di
nuovo l'intera voce.

## Il menu della riga {#the-row-menu}

Fai clic destro su una riga. Se la riga non è selezionata, diventa la
selezione; altrimenti il menu agisce su ogni riga selezionata. La prima riga
mostra il titolo del brano quando il menu è per un brano, e quante tracce sono
selezionate quando è per più d'una.

| Voce | Azione |
|---|---|
| Aggiungi a ▸ | Una playlist: i brani vengono aggiunti in fondo |
| Aggiungi a un cart ▸ | Un cart della pagina mostrata della cartwall: i brani riempiono il cart scelto e i cart vuoti successivi della pagina, nell'ordine della griglia. I brani che non entrano vengono lasciati fuori e un avviso li conta |
| Sposta nella categoria ▸ | Una categoria, o Senza categoria |
| Tipo ▸ | Musica, Jingle, Effetto, Pubblicità o Voce |
| Modifica i tag… | L'editor dei tag (un brano) |
| Loudness… | Il popup del loudness (un brano) |
| Rianalizza | Analizza di nuovo i brani |
| Ripristina i marker automatici | Scarta i marker manuali |
| Mostra la posizione del file | Apre la cartella del file (un brano) |
| Rimuovi dalla libreria… | Rimuove i brani dalla libreria, dopo aver chiesto (sotto) |

## Preascolto {#pre-listening}

La barra sotto la barra del titolo preascolta il brano selezionato
sull'uscita **Libreria** propria della libreria (scelta in
[Impostazioni → Uscite audio](settings.md#audio-outputs)), per esempio le
cuffie. Un telefono preascolta invece dal suo elenco di brani (vedi
[Su un telefono](#on-a-phone)).

La barra mostra:

- **Preascolto**, **Metti in pausa il preascolto** e **Ferma il
  preascolto**;
- il grande tempo rimanente del brano, il suo titolo, l'artista e la
  categoria;
- un piccolo meter a destra. Legge solo questo preascolto, mai il meter di un
  lettore o della cartwall; fai clic su di esso per riavviarne il massimo;
- la forma d'onda dell'intero file, con i suoi marker, e una riga
  `trascorso / totale` sotto di essa.

Preascolto fa partire il brano selezionato dal suo cue-in e lo riproduce fino
alla fine del file. Preascolto su un'anteprima in pausa la riprende, e su una
in riproduzione non fa nulla: Ferma, poi Preascolto, riparte dal cue-in. La
normalizzazione del loudness si applica come sul CUE di un lettore.

La forma d'onda funziona come nella finestra CUE di un lettore: fai clic per
spostarti (lo fa un clic o il rilascio del mouse; il trascinamento no),
ingrandiscila e spostala con la rotella, e modifica i marker e apri
**Loudness…** dal suo menu (vedi
[Modificare i marker](markers-and-mixing.md#editing-markers)). Mentre
l'anteprima è ferma, un clic imposta invece da dove parte il prossimo
Preascolto.

**L'anteprima segue la selezione.** Con più righe selezionate, usa l'ultima
su cui hai fatto clic.

- Mentre suona, selezionare un'altra riga passa a quel brano dal suo cue-in e
  continua a suonare, senza chiedere.
- Mentre è in pausa, il nuovo brano viene caricato in pausa.
- Mentre è ferma, resta ferma e la barra mostra il nuovo brano.
- Non selezionare nulla (nessuna riga selezionata, o la riga selezionata
  filtrata via) lascia com'è un'anteprima in corso.
- Selezionare un brano che non si può riprodurre (il suo file manca o non si
  legge) ferma l'anteprima, e la barra dice che il file non si può
  riprodurre.

**Cosa la ferma.** Ferma; rimuovere il suo brano dalla libreria; chiudere la
finestra (ridurla a icona no). Non suona mai dopo un riavvio.

**Senza un'uscita Libreria** (sul desktop; Android suona invece
sull'uscita predefinita, vedi [Android → Libreria](android.md#library))
Preascolto è attenuato (passa il puntatore per il motivo) e una riga accanto
dice che il preascolto richiede un'uscita CUE della libreria, con un
collegamento che apre Impostazioni → Uscite audio sulla riga **Libreria**. La
barra mostra comunque il brano, la sua forma d'onda e i suoi marker.

**Su un'uscita in onda.** L'uscita Libreria può essere un'uscita in onda (lo
stesso dispositivo e gli stessi canali del Main di un lettore o della
cartwall). Funziona allora come il CUE dei lettori e della cartwall (vedi
[CUE (preascolto)](players.md#cue-pre-listen)): finché lì suona qualcosa,
Preascolto chiede prima (**Preascolta comunque** o **Annulla**), e un
preascolto che non hai confermato si ferma quando qualcosa parte su
quell'uscita.

## Trascinare sulle playlist e sui cart {#dragging-to-playlists-and-carts}

Trascina con il mouse le righe selezionate (o la riga da cui inizi a
trascinare, quando non è selezionata):

- su una **scheda di playlist** di un lettore: i brani vanno in fondo a quella
  playlist;
- su una **tabella di playlist**: i brani vanno dove mostra la linea viola;
- su un **cart**: il cart riceve il primo brano (un cart contiene un solo
  file), sostituendo il suo file quando è fermo. Un cart in riproduzione o in
  CUE rifiuta il rilascio, perché un nuovo file lo fermerebbe in onda.

## Rimuovere i brani {#removing-tracks}

**Rimuovi dalla libreria…** chiede prima: i file non vengono eliminati.
Quando uno qualsiasi dei brani è in uso, una seconda domanda conta le voci di
playlist e i cart che li perdono. Un brano in onda (in riproduzione in un
lettore o sulla cartwall) non può essere rimosso: non viene rimosso nulla e un
avviso dice cosa suona.

**Rimuovi i brani mancanti…** (nel menu della categoria, su qualsiasi
categoria) rimuove i brani di quella categoria e delle sue sottocategorie il
cui file manca e che nessuna voce di playlist o cart usa. Chiede prima, con il
loro numero; nessun file viene toccato. I brani in uso vengono mantenuti, e
una categoria che non ne ha lo dice in un avviso. Usalo per ripulire dopo che
dei file hanno lasciato una cartella sorvegliata: l'applicazione non li
rimuove mai da sola.

## Da una playlist {#from-a-playlist}

Sul desktop, il menu del brano di una riga della playlist ha **Mostra nella
libreria** e **Sposta nella categoria ▸**, dopo **Loudness…**. *Mostra nella
libreria* apre la finestra sulla categoria del brano e ne seleziona la riga;
*Sposta nella categoria* archivia il brano senza aprire la finestra. Per un
brano che non è nella libreria, *Mostra nella libreria* è mostrato
disattivato e *Sposta nella categoria* non è mostrato.

## Su un telefono {#on-a-phone}

Un telefono non ha una finestra della libreria: la sua scheda **Libreria** si
apre sulle categorie (*Tutti i brani*, le tue categorie, *Senza categoria* e
**Musica del dispositivo**), e un tocco su una ne elenca i brani con una
ricerca e **Tutta la libreria**. Spunta i brani per aggiungerli a una
playlist, tieni premuta una riga per il suo menu, preascolta un brano con le
sue 🎧, e aggiungi la musica del dispositivo a una categoria con il **＋**
del suo elenco. Su Android, senza un'uscita Libreria, il preascolto suona
sull'uscita predefinita. Vedi [Android → Libreria](android.md#library).
