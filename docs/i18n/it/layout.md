# Layout

Disponi la finestra principale per il tuo turno: dai ai lettori e alla
cartwall le dimensioni che vuoi, metti la cartwall sotto i lettori o accanto
a essi, nascondi i pannelli che non usi e ingrandisci per un momento uno di
essi. Il layout viene salvato con la sessione e ritorna al prossimo avvio; una
nuova installazione parte con tutti i lettori della stessa larghezza e la
cartwall sotto di essi.

## Mostrare e nascondere i pannelli {#show-and-hide-panels}

Le tessere a sinistra della barra superiore, **Palinsesto** e **Musica** (il
[palinsesto](automation.md#the-schedule-column) e il
[riempitivo musicale](automation.md#the-music-filler), entrambi nascosti
finché non li mostri), **P1** … **Pn** e **Cartwall**, mostrano quali pannelli
sono sullo schermo: una tessera accesa è mostrata. Fai clic su una tessera
per nascondere o mostrare il suo pannello. Con molti lettori, le tessere
scorrono di lato (usa la rotella del mouse su di esse) e il pulsante del
layout resta in vista. Gli altri pannelli prendono lo spazio che resta
libero, e un lettore mostrato di nuovo torna alla sua larghezza.

Nascondere toglie soltanto un pannello dallo schermo. Un lettore nascosto
continua a suonare, e il suo tasto numerico, il MIDI, l'API remota, la sua
finestra CUE e l'automazione lo controllano ancora. Una tessera mostra lo
stato del suo pannello, sia che il pannello sia mostrato o nascosto: un punto
rosso mentre un lettore sta suonando (o la cartwall mentre suona un cart) e
due barre ambra, come un pulsante Pausa, mentre un lettore è in pausa.

Nascondere un lettore ne cancella il brano selezionato, così **Delete** non
può rimuovere un brano che non vedi. I file rilasciati sulla finestra lontano
da un elenco vanno all'ultimo lettore che hai usato, o al primo lettore
mostrato; con tutti i lettori nascosti vengono rifiutati, e la barra di stato
lo dice.

La tessera **Libreria**, dopo di esse, apre e chiude la [finestra della
libreria](library.md), che fluttua sopra i pannelli; non è un pannello,
quindi la regola dell'ultima tessera accesa qui sotto non vale per essa. Non
è mostrata su un telefono.

Almeno un pannello resta mostrato: l'ultima tessera accesa non si può
cliccare. Mentre la cartwall è nascosta, il tasto che la comprime o la
espande non fa nulla; fai clic su **Cartwall** per mostrarla.

## Ridimensionare i lettori {#resize-the-players}

Trascina lo spazio tra due lettori per spostare larghezza dall'uno all'altro.
Gli altri lettori non si muovono, e nessun lettore diventa più stretto di
quanto servano i suoi controlli. Fai doppio clic sullo spazio per dare di
nuovo a ogni lettore la stessa larghezza.

Quando la finestra è troppo stretta per tutti i lettori mostrati alla loro
dimensione minima, la riga dei lettori scorre di lato, come ha sempre fatto.

## Spostare e ridimensionare la cartwall {#move-and-resize-the-cartwall}

La cartwall sta sotto i lettori, oppure a sinistra o a destra della finestra
per tutta la sua altezza: scegli **Cartwall in basso**, **Cartwall a
sinistra** o **Cartwall a destra** nel [menu del layout](#the-layout-menu).
Sotto i lettori è alta quanto servono le sue righe, al massimo sei decimi
dell'altezza tra la barra superiore e la barra di stato; accanto ai lettori
all'inizio prende tre decimi della larghezza.

La cartwall non è mai più stretta della sua intestazione (**Ferma tutto**, il
suo titolo e una scheda di pagina) né delle sue colonne di pulsanti alla loro
dimensione minima. Quando la finestra è troppo stretta per essa e per tutti i
lettori mostrati alla loro dimensione minima, la riga dei lettori scorre di
lato.

Trascina lo spazio tra la cartwall e i lettori per ingrandirla o
rimpicciolirla: su o giù quando è sotto i lettori, di lato quando è accanto a
essi. I lettori mantengono lo spazio che i loro controlli richiedono. Fai
doppio clic sullo spazio per tornare alla dimensione iniziale: alta quanto
servono le sue righe sotto i lettori, tre decimi della larghezza accanto a
essi. La dimensione viene salvata con il layout; spostare la cartwall la fa
ripartire dalla sua dimensione iniziale.

Dopo che hai ridimensionato la cartwall, o mentre sta accanto ai lettori, i
suoi pulsanti si allargano per riempirla; i nomi e i tempi mantengono la loro
dimensione, al centro di ogni pulsante. Un nome va a capo tra le sue parole;
una parola troppo lunga per il suo pulsante termina con dei puntini di
sospensione (…) invece di spezzarsi su più righe. Quando la cartwall è più
piccola di quanto richiedano le sue righe e colonne con i pulsanti più
piccoli, scorre.

Fai clic su **CARTWALL** per comprimere la cartwall. Sotto i lettori resta
solo la sua intestazione; accanto ad essi diventa una striscia stretta alta
quanto la finestra, con una freccia per espanderla di nuovo e, mentre un cart
suona, un punto rosso. Espansa, riprende la sua dimensione. Mentre è
compressa, lo spazio accanto a essa non fa nulla.

## Ingrandire un pannello {#maximize-a-panel}

L'intestazione di ogni lettore, e quella della cartwall, termina con due
piccoli pulsanti: **Nascondi** (un meno) e **Ingrandisci** (angoli che
puntano all'esterno; **Ripristina**, angoli che puntano all'interno, sul
pannello ingrandito). Passa il puntatore per il nome. Sono attenuati mentre
il layout è bloccato, **Nascondi** sull'ultimo pannello mostrato, e
**Ingrandisci** con un solo pannello mostrato e su una cartwall compressa. Le
stesse due azioni sono nel menu di un clic destro sullo sfondo
dell'intestazione (non su uno dei suoi pulsanti):

- **Ingrandisci** dà a quel pannello tutto lo spazio: gli altri pannelli si
  riducono alla loro dimensione minima. Vale solo per il momento e non viene
  salvato. **Ripristina** nello stesso menu lo termina, e lo terminano anche
  nascondere o mostrare un pannello, bloccare il layout, spostare la
  cartwall, cambiare il numero di lettori e trascinare o fare doppio clic su
  uno spazio tra i pannelli; le dimensioni di prima ritornano. Una cartwall
  compressa non si può ingrandire, e comprimere una cartwall ingrandita lo
  termina.
- **Nascondi** nasconde il pannello, come fa la sua tessera nella barra
  superiore.

## Il menu del layout {#the-layout-menu}

Il pulsante del layout, a destra delle tessere, apre un menu con:

- **Cartwall in basso**, **Cartwall a sinistra**, **Cartwall a destra**:
  dove sta la cartwall;
- **Blocca il layout**: finché è spuntato, nulla cambia il layout per
  errore: le tessere, le altre voci di questo menu, gli spazi tra i pannelli
  e **Ingrandisci** e **Nascondi** delle intestazioni non fanno nulla. La
  riproduzione, e comprimere o espandere la cartwall, funzionano come al
  solito. Il blocco viene salvato con il layout;
- **Lettori di uguale larghezza**: ogni lettore della stessa larghezza;
- **Reimposta il layout**: torna al layout di una nuova installazione.
