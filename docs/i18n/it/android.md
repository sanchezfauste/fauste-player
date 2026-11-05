# Android

Fauste Player funziona anche su telefoni e tablet Android (Android 10 o
successivo), sullo stesso nucleo di playout del desktop: i lettori, le
playlist, la cartwall, i marker, la normalizzazione del loudness e ogni
regola di riproduzione funzionano allo stesso modo.

## Installazione {#install}

Una release che include l'app ha `fauste-player-<version>-android.apk` nella
sua pagina **Releases** (con il suo `.sha256`). Aprilo sul telefono e
consenti al browser o al file manager di installare app quando Android lo
chiede. Per aggiornare, installa il nuovo APK allo stesso modo: le tue
playlist e impostazioni restano. L'app non è ancora su Google Play.

## Primo avvio {#first-start}

- **Accesso alla musica.** Consenti l'accesso alla musica (e alle
  notifiche, su Android 13 e successivi). Senza, il browser musicale resta
  vuoto.
- **Il playout continua.** Mentre un lettore o un cart suona, resta visibile
  una notifica e l'audio prosegue con lo schermo spento o l'app in secondo
  piano. Come sul desktop, nulla va in onda da solo dopo un avvio o un
  crash.
- **Indietro e chiusura.** Indietro chiude prima ciò che è aperto (il
  browser musicale, le Impostazioni, Informazioni, un lettore aperto, una
  pagina della scheda Libreria). Quando non resta nulla da chiudere, Indietro
  mette via l'app mentre qualcosa suona (un lettore, un cart o un preascolto
  CUE) e continua a riprodurre; quando nulla suona, salva e chiude l'app.
  Far scorrere via l'app dal selettore delle app fa lo stesso: uno scorrimento
  non ferma mai ciò che sta suonando, e l'app si chiude quando tace.
- **L'audio di altre app.** Quando un'altra app prende l'audio, Fauste Player
  continua a riprodurre e lo dice nella barra di stato in basso.

## Lo schermo {#the-screen}

Su un **telefono** la barra superiore ha quattro schede, **Diretta**,
**Cartwall**, **Playlist** e **Libreria**, mostrate come icone (quella scelta
anche con il suo nome quando c'è spazio), poi il **lucchetto** e **⋮** (Altro:
Impostazioni e Informazioni). L'orologio compare quando c'è spazio. Con il
telefono di lato, lo schermo Playlist mette le sue etichette dei lettori (P1,
P2…) nella barra superiore, e la barra di stato in basso compare solo quando
ha qualcosa da dire.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Diretta su un telefono: due lettori come schede, una intera e l'inizio della successiva, e la cartwall sotto">
<img src="../../images/android/2-carts.png" width="180" alt="Cartwall su un telefono: la pagina General in verticale, due cart per riga">
<img src="../../images/android/3-playlists.png" width="180" alt="Playlist su un telefono: le etichette da P1 a P4, le schede della playlist, l'elenco dei brani e la scheda del lettore in basso">
<img src="../../images/android/4-music.png" width="180" alt="La musica del dispositivo su un telefono: la cartella Demo con tre brani spuntati e la barra per aggiungerli alla playlist Morning di P1">
</p>

- *Diretta:* ogni lettore come una scheda, e la cartwall. Ogni scheda ha,
  dall'alto in basso, il nome del lettore, lo stato, Single/Cont e CUE; la
  copertina, il titolo e l'artista e il brano successivo; la forma d'onda (un
  tocco si sposta); poi il tempo trascorso e la durata del brano con il tempo
  rimanente a grandi cifre sopra il trasporto (Play/Successivo, Precedente,
  Pausa, Stop, Riavvia, Stop in dissolvenza, Stop dopo il brano attuale) con
  pulsanti grandi quanto un dito, e un meter sottile con il fader del volume
  (il livello del fader compare sotto di esso). Il tempo rimanente termina
  dove terminano i pulsanti, più piccolo per un brano lungo. Sulla maggior
  parte dei telefoni il meter mostra la sua scala alla sua sinistra, e su uno
  più largo (circa 406 dp e oltre) il fader mostra anche i suoi livelli; su un
  telefono stretto (sotto circa 384 dp) nessuno dei due la mostra, e i
  pulsanti diventano un po' più piccoli per starci. I pulsanti funzionano
  mentre un altro dito tiene il fader, e un tocco vale anche se il dito rotola
  un po' o resta premuto un momento. In verticale le schede stanno in alto,
  una intera e l'inizio della successiva (scorri per vederne altre), e la
  cartwall sotto. Di lato ogni lettore è una riga a sinistra (il suo nome e
  stato, il titolo, il tempo rimanente, Play/Successivo, Pausa e Stop; tocca
  la riga altrove per aprire il lettore su Playlist) e la cartwall è a
  destra. Tocca **CARTWALL** nella sua intestazione per ripiegarla: resta solo
  l'intestazione, in basso (le schede delle pagine e *Ferma tutto*
  funzionano ancora), e i lettori prendono lo spazio; toccala di nuovo per
  riaprirla. La scheda Cartwall mostra sempre l'intera cartwall. Una pagina di
  cart troppo larga per lo spazio si mostra in meno colonne, nello stesso
  ordine.
- *Cartwall:* la pagina di cart mostrata, che riempie lo schermo. Scorri di
  lato per mostrare la pagina successiva o la precedente. In verticale, una
  pagina più larga che alta viene ruotata (8 × 2 si mostra come 2 × 8), nello
  stesso ordine.
- *Playlist:* le etichette in alto (P1, P2…) scelgono il lettore; sotto di
  esse ci sono le sue schede di playlist (+ ne crea una nuova), l'elenco dei
  brani e, in basso (a destra quando il telefono è di lato), la scheda del
  lettore con una forma d'onda più grande per preparare un brano (diventa più
  bassa quando manca spazio, così i pulsanti si vedono sempre). Una pressione
  prolungata su una riga apre il suo menu, che ha anche **Sposta su**,
  **Sposta giù** e **Sposta in un altro lettore** (il brano va subito dopo il
  brano successivo di quel lettore, nella playlist che mostra; il brano in
  onda resta). Trascina una riga dalla sua maniglia ≡ per spostarla. Il piè
  di pagina ha **Aggiungi** e, dietro **⋯**, **Azzera**. Una pressione
  prolungata sulla forma d'onda imposta un marker in quel punto (cue-in, fine
  dell'intro, inizio dell'outro, MIX, cue-out) o ne rimuove uno che hai
  impostato; i marker si modificano solo qui, non su Diretta.
- *Libreria:* la libreria dei brani e la musica del dispositivo (vedi
  [Libreria](#library)).

Indietro da un altro schermo va a Diretta; su Libreria torna prima all'elenco
delle categorie (su Musica del dispositivo, prima su di una cartella).

Quando nulla si muove sullo schermo, l'app lo disegna una volta al secondo
(per l'orologio) per risparmiare la batteria; un meter che sta ancora
scendendo dopo una pausa o uno stop continua a disegnare finché il meter non
è sceso.

**Il lucchetto** protegge dai tocchi involontari durante uno spettacolo.
Bloccato (il lucchetto chiuso e acceso), i fader del volume mostrano solo il
livello e le forme d'onda non si spostano né modificano i marker; appaiono
attenuati. Un pizzico e un trascinamento ingrandiscono e spostano comunque
una forma d'onda, e Play, il trasporto, i cart e le playlist funzionano
sempre. Il blocco resta dopo un riavvio e viene salvato con la sessione,
quindi caricare un'altra sessione prende il blocco di quella sessione;
l'API remota e il MIDI non sono bloccati.

Un trascinamento che parte da un fader del volume lo muove; uno che parte
altrove scorre, così scorrere non cambia mai un livello in onda. Un tocco su
una forma d'onda si sposta lì, un pizzico a due dita su di essa ingrandisce
attorno alle tue dita, un trascinamento a un dito sposta una vista
ingrandita e **Vista completa** mostra di nuovo l'intero brano. In una
playlist un dito scorre; per spostare un brano, usa la sua maniglia ≡ o il
menu della riga.

Su un **tablet** il layout è quello del desktop (vedi [Layout](layout.md)),
con bersagli più facili da colpire.

Una pressione prolungata apre ciò che un clic destro apre sul desktop: il
menu della riga, i menu delle intestazioni, il menu di un cart (**Modifica…**
sceglie il file di un cart vuoto). Su un telefono, le Impostazioni riempiono
lo schermo ed elencano le loro sezioni; toccane una per aprirla, con il nome
di ogni impostazione sopra il suo controllo. La freccia o Indietro torna
all'elenco, e di nuovo Indietro (o **Chiudi**) chiude le Impostazioni.
Modificare un cart apre direttamente la sua sezione.

Informazioni → **Licenze di terze parti** apre in un browser o in un
visualizzatore HTML le note delle librerie con cui è costruita l'app.

## Libreria {#library}

Su un **telefono** la scheda **Libreria** si apre su un elenco di categorie:
**Tutti i brani**, le tue categorie (ogni sottocategoria sotto la sua
principale, con il suo colore e quanti brani contiene), **Senza categoria** e
**Musica del dispositivo**. La libreria è la stessa che mostra la [finestra
della libreria](library.md) del desktop: ogni file aggiunto a una playlist, a
un cart o alla libreria ne fa parte. Le categorie si creano, si rinominano, si
ricolorano e si eliminano in **Impostazioni → Libreria**.

**I brani di una categoria.** Tocca una categoria per aprire i suoi brani
(con quelli delle sue sottocategorie). La freccia nella sua intestazione, o
Indietro, torna alle categorie. Passare a un'altra scheda e tornare ritrova
l'elenco come l'avevi lasciato. Aprire una categoria dalle categorie parte con
una ricerca vuota. La ricerca filtra l'elenco per titolo, artista, album e
gli altri tag che cerca la finestra della libreria; **Tutta la libreria**
cerca in ogni categoria. Ogni riga mostra il titolo (o il nome del file),
l'artista, la durata e i segni della finestra per un file mancante,
illeggibile o non ancora analizzato.

Tocca le righe per spuntarle. Finché qualcosa è spuntato, in basso compare la
barra di Musica del dispositivo: quanti, il lettore e la playlist a cui
vanno (toccala per sceglierne un'altra), **Aggiungi alla fine** e **Aggiungi
dopo il successivo**, come su Musica del dispositivo. I brani vanno
nell'ordine in cui li hai spuntati, e aggiungere cancella le spunte. Tornare
alle categorie le cancella anch'esso.

**Il menu della riga.** Una pressione prolungata su una riga apre il suo menu:
**Aggiungi a ▸** (la playlist di un lettore, in fondo), **Preascolta**,
**Sposta nella categoria ▸**, **Rianalizza** e **Rimuovi dalla libreria…**,
che chiede come sul desktop (vedi
[Rimuovere i brani](library.md#removing-tracks)). Finché ci sono righe
spuntate, una pressione prolungata su un'altra riga spunta anche quella e il
menu agisce su ogni riga spuntata, nell'ordine in cui le hai spuntate;
**Preascolta** non viene allora offerto. Le spunte restano dopo l'azione del
menu.

**Preascolto.** Le 🎧 a destra di una riga riproducono il suo brano dal suo
cue-in sull'uscita di preascolto della libreria; le 🎧 di un'altra riga
passano a quel brano. Una striscia sottile sopra l'elenco mostra il titolo,
il tempo rimanente, **Metti in pausa il preascolto**/**Preascolto** e **Ferma
il preascolto** mentre suona (o è in pausa), e un suggerimento altrimenti:
mantiene sempre il suo posto, così le righe non si spostano quando un
preascolto parte o si ferma. Le 🎧 sono attenuate quando il file manca o non
si può riprodurre. L'anteprima si ferma quando il suo brano lascia la
libreria e quando lasci la scheda Libreria (per un'altra scheda, o con
Indietro); sulle categorie o su Musica del dispositivo continua, ma la sua
striscia compare solo sull'elenco dei brani.

A differenza del desktop, Android (un telefono o un tablet) riproduce il
preascolto della libreria sull'uscita predefinita quando non è scelta alcuna
uscita per esso: in **Impostazioni → Uscite audio → Libreria** la scelta vuota
si legge *Predefinito di sistema*, e il suo **Prova Cue** suona lì. Scegli lì
un'altra uscita (per esempio delle cuffie con filo) per preascoltare lontano
dall'onda. L'uscita predefinita di solito è anche quella dei lettori e della
cartwall, quindi il preascolto può sentirsi in onda: finché su quell'uscita
suona qualcosa, le 🎧 chiedono prima (**Preascolta comunque** o **Annulla**),
e un preascolto che non hai confermato si ferma quando lì parte qualcosa,
come il CUE di un lettore (vedi [CUE (preascolto)](players.md#cue-pre-listen)).

**Aggiungere la musica del dispositivo a una categoria.** Il **＋**
nell'intestazione dell'elenco dei brani apre Musica del dispositivo per quella
categoria. La sua barra ha allora solo **Aggiungi a <categoria>** (**Aggiungi
a Senza categoria** da *Tutti i brani* o *Senza categoria*), che aggiunge i
file spuntati alla libreria in quella categoria e torna all'elenco; un file
già nella libreria vi entra se non ha una categoria e mantiene quella che ha
altrimenti, come sul desktop. Musica del dispositivo mantiene le sue spunte
tra una visita e l'altra, quindi vengono aggiunti anche i file spuntati in
una visita precedente: controlla prima il conteggio sulla barra. **Torna
all’elenco**, in alto, o Indietro torna all'elenco senza aggiungere. Musica
del dispositivo aperta dalle categorie ha invece **Torna alle categorie** in
alto.

**La cartella del dispositivo di una categoria.** Una categoria può prendere
i suoi file da una cartella della musica del dispositivo. Nel suo menu
(pressione prolungata su di essa in **Impostazioni → Libreria**, o nella
finestra della libreria su un tablet), **Usa una cartella del dispositivo…**
apre Musica del dispositivo per scegliere la cartella (**Usa questa
cartella**); il menu della categoria poi la nomina, sopra le sue voci di
cartella. Nulla sorveglia la cartella: quando hai copiato lì nuova musica,
tocca la **↻** nell'intestazione dell'elenco dei brani della categoria (o
**Rileggi la cartella del dispositivo** nel suo menu). Ogni file che Musica
del dispositivo elenca nella cartella e nelle sue sottocartelle viene aggiunto
alla categoria, con le regole qui sopra (un file già in un'altra categoria la
mantiene), e un avviso dice quanti file sono stati letti. I file che hanno
lasciato la cartella restano nella libreria; **Rianalizza** segna come
mancanti quelli il cui file è sparito. **Cambia cartella del dispositivo…** e
**Dimentica la cartella del dispositivo** sono nello stesso menu; una cartella
appartiene a una sola categoria.

### Musica del dispositivo {#device-music}

**Musica del dispositivo** mostra i file audio del dispositivo per
**Cartelle** (con il percorso sopra l'elenco; tocca una sua parte per
risalire), **Artisti** o **Album**, e una ricerca su titolo, artista, album e
nome del file. Ogni riga mostra il titolo, l'artista e la durata; tocca le
righe per spuntarle. Finché qualcosa è spuntato, una barra in basso mostra
quanti, il lettore e la playlist a cui vanno (toccala per sceglierne
un'altra), **Aggiungi alla fine** e, quando il brano successivo di quel
lettore è in quella playlist, **Aggiungi dopo il successivo**. I brani vanno
nell'ordine in cui li hai spuntati. Un file del dispositivo non ha
preascolto.

**Aggiungi** nel piè di pagina di una playlist, il file di un cart e
Impostazioni → Playlist → Cartella della musica aprono Musica del dispositivo
su un telefono (su un tablet, la finestra del **browser musicale**, per
cartella). Spunta i file e tocca **Aggiungi**, tocca un file per un cart, o
**Usa questa cartella**; **Annulla** o Indietro torna dov'eri. Copia la musica
sul dispositivo (per esempio nella sua cartella *Music*) per vederla lì.
Fauste Player legge soltanto la tua musica: i tag e le copertine non vengono
scritti, e le playlist e le pagine di cart non vengono ancora importate né
esportate su Android.

Su un **tablet** la libreria è la [finestra della libreria](library.md) del
desktop, aperta dalla barra superiore.

## Uscite {#outputs}

Impostazioni → Uscite audio elenca le uscite del telefono per tipo: l'altoparlante,
le cuffie con filo, un'interfaccia USB, un altoparlante Bluetooth («JBL Flip
(Bluetooth)»). Ogni lettore, la cartwall e i loro CUE ne scelgono una, come
sul desktop, e così fa il preascolto della libreria (vedi
[Libreria](#library)). Un altoparlante Bluetooth spento e riacceso viene
ritrovato tramite la sua rotta; mentre è assente i suoi lettori si comportano
come con un dispositivo mancante sul desktop. Se due uscite possano suonare
nello stesso momento dipende dal telefono.

## Non su Android {#not-on-android}

Le superfici di controllo MIDI, il controllo remoto, la modifica di tag e
copertine, e i file di playlist e di pagine di cart sono per ora funzioni del
desktop. Una tastiera collegata a un tablet funziona come sul desktop (vedi
[Tastiera](keyboard.md)).
