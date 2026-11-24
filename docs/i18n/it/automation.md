# Automazione

L'automazione tiene la stazione in onda quando nessuno è in studio. Questa
versione ha il **riempitivo musicale**, un lettore che sceglie la musica da
solo, secondo le regole che imposti, e non si ferma mai per mancanza di una
lista, e la colonna **Palinsesto**, che esegue i blocchi a orario del giorno,
creati a partire da modelli nelle Impostazioni.
I lettori che usi già (P1 … Pn) non vengono toccati: restano tuoi per i
programmi che prepari e mandi in onda in diretta.

## La colonna Palinsesto {#the-log-column}

La colonna **Palinsesto** riproduce i **blocchi** del giorno: gruppi di
elementi con un'ora di inizio (un identificativo orario, spot, un programma).
Mostrala dalla tessera **Palinsesto** della barra superiore, come mostri un
lettore.

![La colonna Palinsesto in funzione, accanto alla colonna Riempitivo musicale: i blocchi del giorno per ora di inizio, ciascuno con il suo identificativo della stazione (l'orologio lo segna a orario fisso), gli spot e la rotazione che il riempitivo musicale riproduce dopo; il brano del riempitivo musicale in onda e i successivi](../../images/guide/automation-columns.png)

Il suo elenco sembra una playlist, raggruppato per blocchi. Ogni blocco inizia
con una riga di intestazione: la sua ora di inizio, quanti elementi ha e
quanto durano, "modificato per questo giorno" quando l'hai modificato oggi e,
in ambra, di quanto sconfina nel blocco successivo. Poi vengono i suoi
elementi, con l'**Ora** in cui è previsto che inizi ciascuno. L'elenco mostra
oggi, l'ultimo blocco di ieri e il primo di domani. L'elemento in onda e il
successivo sono segnati come su un lettore; gli elementi già andati in onda
sono attenuati, e un elemento che non è andato in onda (**non trasmesso**) è
barrato. Un orologio segna un elemento a **orario fisso**, che va in onda solo
alla sua ora. Un **brano di categoria** mostra il brano scelto per lui appena
diventa il successivo (o viene dopo il successivo nel suo blocco), con la
categoria dopo: è il brano che andrà in onda. Se la categoria non ha nessun
brano che possa suonare (nessuno le è assegnato, o mancano i loro file), la
riga lo dice in ambra, e l'elemento non verrà trasmesso. Un elemento il cui
file risulta illeggibile quando sta per suonare viene saltato e segnato anche
lui come non trasmesso. I brani si assegnano a una categoria nella libreria.
La riga del **riempitivo musicale** dice fino a quando suona, e appare in onda
mentre il riempitivo musicale suona per il Palinsesto. Un successivo la cui
ora è passata mostra l'ora in ambra. Trascina la linea tra due intestazioni di
colonna per cambiare la larghezza delle colonne, come in una playlist; le
finestre di modello e di giorno usano le stesse larghezze.

### Modificare il palinsesto di oggi {#changing-todays-log}

Il Palinsesto si modifica come una playlist, e la modifica vale solo per quel
giorno: il modello resta com'è.

- I pulsanti in fondo alla colonna: **+ Brano…** apre la libreria, per
  trascinare brani nell'elenco; **+ Brano di categoria…** e
  **+ Riempitivo musicale (rotazione)…** chiedono quale e lo aggiungono subito
  sotto la riga selezionata (fai clic su una riga per selezionarla; seleziona
  un'intestazione di blocco per aggiungere all'inizio di quel blocco), oppure,
  senza nulla di selezionato o sotto una riga che non può più cambiare, alla
  fine del prossimo blocco; **+ Blocco orario…** chiede un'ora, con la prima
  libera già inserita. Quando la colonna è troppo stretta per tutti, quelli che
  non ci stanno sono dietro **⋯**.
- Trascina brani dalla libreria, o file audio dal tuo gestore di file, su una
  riga per metterli prima di essa (o dopo, sulla metà inferiore della riga,
  come in una playlist: è così che vanno alla fine di un blocco), e trascina
  righe all'interno dello stesso giorno per spostarle. La riga che aggiungi
  viene selezionata e portata in vista, così la successiva che aggiungi va
  sotto.
- Clic destro su una riga per **Orario fisso** e **Togli dal palinsesto di
  oggi**. Fai clic (o clic destro) su un'intestazione di blocco per aggiungere
  un **Brano…**, un **Brano di categoria…** o un **Riempitivo musicale
  (rotazione)…** alla fine di quel blocco, oppure per **Cambia orario…**,
  **Rimuovi blocco** e, su un blocco modificato, **Torna al modello**.
- Mentre trascini sopra un'intestazione di blocco, la linea sotto di essa
  indica che il rilascio va all'inizio di quel blocco; per mettere righe alla
  fine del blocco sopra, rilasciale sulla metà inferiore della sua ultima riga.
- Ciò che è già andato in onda, l'elemento in onda e i giorni passati non si
  possono modificare. Una modifica al blocco in onda ha effetto dopo
  l'elemento in onda.

La colonna Palinsesto ha l'intestazione e i pulsanti di un lettore, e
funzionano come su un lettore:

- **Riproduci** riproduce l'elemento segnato come successivo e avvia
  l'automazione: ogni blocco suona allora alla sua ora, e il riempitivo
  musicale suona tra un blocco e l'altro. Quando il successivo l'ha scelto
  l'orologio e il suo blocco inizia più tardi, la musica suona fino ad allora.
  Niente parte da solo: dopo ogni avvio dell'applicazione, l'automazione è
  ferma finché non premi Riproduci.
- **Pausa**, **Stop** e **Stop in dissolvenza** agiscono su ciò che è in onda
  per il Palinsesto (il suo elemento, o il brano del riempitivo musicale tra i
  blocchi) e fermano l'automazione. Riproduci dopo Pausa riprende da dove era;
  durante la pausa il successivo non si sposta, e alla ripresa il blocco
  successivo mantiene la sua ora. Riproduci dopo uno Stop durante la musica
  rimette musica, e il blocco successivo inizia alla sua ora.
- **Stop alla fine** lascia finire ciò che è in onda, e poi l'automazione si
  ferma. Se sta suonando la musica e arriva prima l'ora di un blocco, il
  blocco inizia puntuale e l'automazione si ferma dopo il suo primo elemento.
- **Brano precedente** non fa nulla sul Palinsesto.
- **SINGLE** rende manuale il Palinsesto: Riproduci riproduce l'elemento
  successivo, e si ferma dopo di esso; nessun blocco parte da solo.

Mentre il riempitivo musicale suona tra i blocchi, l'intestazione del
Palinsesto dice **Riempitivo musicale · *rotazione*** (la rotazione che
riproduce) e i suoi pulsanti continuano a funzionare; il brano, il suo
indicatore di livello e la sua forma d'onda sono solo nella colonna
Riempitivo musicale.

Quando l'automazione si ferma dopo un elemento, il successivo resta
sull'elemento sotto quello che ha suonato, e Riproduci lo riproduce,
qualunque sia l'ora: scegli tu. Quando si ferma durante la musica, Riproduci
rimette musica fino al blocco successivo, a meno che tu non abbia scelto un
altro successivo. Per cambiare ciò che va in onda dopo, fai doppio clic su un
elemento, oppure fai clic destro e scegli **Riproduci dopo**. **Torna
all’orologio**, nello stesso menu, imposta come successivo ciò che dice
l'ora.

## Modelli e giorni {#templates-and-days}

Il Palinsesto riproduce ogni giorno a partire da un **modello**: una giornata
di blocchi. Una nuova installazione ne ha uno, "Base", per tutti i giorni. I
modelli si creano in Impostazioni → **Automazione**, sotto **Modelli e
giorni** (non su Android); la colonna Palinsesto modifica solo oggi, come
sopra.

![Impostazioni, Automazione, Modelli e giorni: i modelli Base e Weekend con Modifica…, Nuovo modello, Duplica, Elimina e il nome; la settimana tipo, Base dal lunedì al venerdì e Weekend il sabato e la domenica; il mese con oggi scelto, le date con modifiche in ambra, il modello della data e Modifica giorno…](../../images/guide/settings-plans.png)

- **Modelli.** Scegline uno e premi **Modifica…** (o fai doppio clic) per
  aprirlo; **Nuovo modello**, **Duplica** ed **Elimina** sono accanto, e il
  suo nome sotto.
- **Settimana tipo.** Quale modello usa ogni giorno della settimana.
- **Giorni.** Scegli una data nel calendario (le date con modifiche sono in
  ambra). Una data può usare un modello diverso da quello del suo giorno della
  settimana (un festivo di giovedì può usare quello del fine settimana).
  **Modifica giorno…** apre quella data, per modificarne i blocchi solo per
  quella data; le date passate non si possono modificare.

Un modello o una data si apre in una finestra propria, che chiude le
Impostazioni. È lo stesso elenco della colonna Palinsesto, e si modifica allo
stesso modo: i pulsanti in fondo (Brano…, Brano di categoria…, Riempitivo
musicale (rotazione)…, Blocco orario…, che aggiungono sotto la riga
selezionata), brani trascinati dalla libreria (o file dal tuo gestore di file)
su una riga, righe trascinate per spostarle, il menu del clic destro delle
righe e il menu delle intestazioni di blocco. Non suona mai, e può restare
aperta accanto alla libreria. La finestra di oggi mostra ciò che è andato in
onda e ciò che suona, come il Palinsesto; la finestra di un giorno si chiude a
mezzanotte, quando quel giorno non può più cambiare.

![Un modello aperto nella sua finestra: i blocchi del modello Base per ora di inizio, ciascuno con il suo identificativo della stazione, i suoi spot e il riempitivo musicale dopo, e Aggiungi in fondo](../../images/guide/template-popup.png)

Un **brano di categoria** in un modello riproduce un brano di quella
categoria, ogni volta uno diverso. L'elemento **Riempitivo musicale** sceglie
la rotazione che il riempitivo musicale riproduce dopo il blocco (la
rotazione predefinita quando il blocco non ne ha una). Quando gli elementi di
un blocco durano più del tempo fino al blocco successivo, la sua
intestazione dice di quanto, in ambra; puoi comunque lasciarlo così.

Una modifica a una data è segnata con "modificato per questo giorno" sul suo
blocco, e **Torna al modello**, nel menu dell'intestazione del blocco, la
annulla. Le modifiche successive al modello continuano a raggiungere i
blocchi che non hai modificato in quella data. Una modifica a un modello fatta
mentre uno dei suoi blocchi è in onda lascia quel blocco com'è per oggi
(mostra "modificato per questo giorno"); la modifica lo raggiunge dal giorno
dopo.

## Il riempitivo musicale {#the-music-fill}

Il riempitivo musicale è la colonna **Riempitivo musicale**. È nascosta finché
non la mostri: fai clic sulla tessera **Riempitivo musicale** a sinistra della
barra superiore. Si trova a sinistra di P1.

Ha l'aspetto e il funzionamento di un lettore: il grande orologio, la forma
d'onda, i pulsanti di trasporto, il fader e l'elenco dei brani. La differenza
è l'elenco:

- **si riempie da solo** a partire da una **rotazione** (sotto), sempre con
  circa mezz'ora di anticipo (*anticipo*, in Impostazioni → Automazione);
- **si sfoltisce da solo**: conserva gli ultimi 50 brani che ha riprodotto e
  rimuove quelli più vecchi, così può funzionare per settimane;
- prima di un blocco del palinsesto può **sostituire i brani che ha scelto
  lui** con altri la cui durata si adatta al tempo che resta (vedi sotto); i
  brani che hai aggiunto tu non vengono mai toccati;
- puoi comunque **modificarlo a mano**: trascina brani dalla libreria,
  riordina le righe, rimuovi un brano che non vuoi. Una riga che aggiungi è
  segnata con una mano e non viene mai sostituita; una riga che rimuovi non
  conta come riprodotta.

Le sue uscite si impostano in Impostazioni → Uscite audio, nella riga
**Riempitivo musicale**, come per qualsiasi lettore. Non ha un numero, né
scorciatoie da tastiera o associazioni MIDI, e il controllo remoto non lo
vede.

Premi **Riproduci** per avviarlo e **Stop alla fine** per lasciar finire il
brano in onda e fermarsi. Come ogni lettore, non parte mai da solo, nemmeno
dopo un riavvio. Mentre suona per il Palinsesto tra i blocchi, fermarlo o
metterlo in pausa ferma anche l'automazione, e il suo Stop alla fine ferma
l'automazione quando il brano finisce; se arriva prima l'ora di un blocco, il
blocco suona e il segno viene tolto. Dopo che il Palinsesto è stato messo in
pausa durante la musica, Riproduci nella colonna Riempitivo musicale
riproduce il brano per conto suo: il Riproduci del Palinsesto non lo riprende
più, e il suo Stop non lo ferma più.

Sopra il suo elenco, dove un lettore ha le schede delle playlist, la colonna
Riempitivo musicale mostra la **rotazione** che riproduce. Scegline un'altra
lì: i brani che aveva in coda vengono sostituiti, quelli che hai aggiunto
restano. Quando la rotazione non trova nulla da riprodurre (le sue categorie
sono vuote, o mancano tutti i file), la barra lo dice.

## Rotazioni {#rotations}

Una **rotazione** è un breve elenco di **passi** che il riempitivo musicale
riproduce a turno, poi di nuovo dal primo. Ogni passo indica da dove vengono i
suoi brani (una categoria della libreria con le sue sottocategorie, o tutti i
brani di un tipo) e quanti ne riproduce ogni volta, come intervallo:

| Passo | Riproduce |
|---|---|
| Successi attuali | 2–3 |
| Identificativi della stazione | 1 |
| Oldies | 0–1 |

Con un intervallo, un identificativo della stazione arriva dopo due brani una
volta e dopo tre la volta successiva. Un intervallo che parte da **0** lascia
che un passo salti alcuni turni: con Oldies a 0–1, un brano oldies suona dopo
l'identificativo della stazione alcune volte e altre no. Il massimo è sempre
almeno 1. Una rotazione è quella **predefinita**: una nuova installazione ne
ha una, **Musica**, che riproduce tutta la tua musica.

Un passo la cui categoria non ha nessun brano che possa suonare (una
categoria vuota, o file mancanti) riproduce al suo turno un brano della
rotazione predefinita (musica qualsiasi, se il passo è della rotazione
predefinita stessa), così gli altri passi mantengono il loro posto: un
identificativo della stazione arriva sempre tra i brani.

Le rotazioni si creano in Impostazioni → Automazione:

![Impostazioni, Automazione: le rotazioni Music (la predefinita) e Hits, la Music selezionata con il suo unico passo su tutta la musica, i suoi conteggi Almeno e Al massimo, e l'inizio di Modelli e giorni](../../images/guide/settings-automation.png)

## Come sceglie {#how-it-chooses}

All'interno di un passo, il riempitivo musicale preferisce i brani che non
vanno in onda da più tempo, con un po' di casualità tra loro perché l'ordine
non diventi mai un ciclo fisso. Tiene inoltre separati:

- **lo stesso brano:** può suonare di nuovo quando sono passate 3 ore o quando
  è andata in onda la metà degli altri brani del suo passo, a seconda di quale
  accade prima;
- **lo stesso artista:** non entro 30 minuti.

Conta tutto ciò che è andato in onda, dal riempitivo musicale, da un lettore o
dalla cartwall; il preascolto no. Quando una piccola categoria non riesce a
rispettare le regole, il riempitivo musicale cede piuttosto che restare in
silenzio: prima sull'artista, poi sul brano. Impostazioni → Automazione ha
tutti i valori, ed è dove crei, rinomini e modifichi le rotazioni (vedi
[Impostazioni](settings.md#automation)).

Prima di un blocco del palinsesto, il riempitivo musicale sceglie anche i suoi
ultimi brani in base alla durata, così la musica finisce in modo pulito: il
brano che sfuma all'inizio del blocco sta ancora suonando all'ora del blocco e
perde al massimo 30 secondi dopo di essa, e non parte nessun brano che
suonerebbe per meno di 45 secondi. Rispetta anche le regole di artista e di
brano ogni volta che può. Quando nessun brano della rotazione può fare
entrambe le cose, suona quello che perde meno, così a volte si possono
sentire alcuni secondi di un brano. Una pausa della musica sposta i brani che
seguono, e il riempitivo musicale li sceglie di nuovo. Il riempitivo musicale
evita anche i brani, e gli artisti, che un blocco riproduce poco dopo. Per ora
entrambi i valori si possono cambiare solo nel file di configurazione
(`automation.max_fade_cut_secs` e `automation.min_song_play_secs`).

Un **Identificativo orario** è un tipo di brano a sé (Libreria → Tipo ▸
Identificativo orario), per l'identificazione della stazione allo scoccare
dell'ora.
