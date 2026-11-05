# Cartwall

La cartwall è il pannello di pulsanti sotto i lettori, o accanto a essi (vedi
[Layout](layout.md#move-and-resize-the-cartwall)). Ogni pulsante, un
**cart**, riproduce all'istante un suono: jingle, effetti, spot. I cart
suonano sulle proprie uscite, indipendentemente dai lettori.

![La cartwall con un cart in riproduzione](../../images/guide/cartwall.png)

## Uso {#using-it}

- **Fai clic su un cart** per lanciarlo. **Fai di nuovo clic** per fermarlo.
- **Trascina i brani dalla [finestra della libreria](library.md)** su un cart
  per metterci il primo (sostituendo il suo file quando è fermo). Un cart in
  riproduzione o in CUE rifiuta il rilascio.
- **Ferma tutto** (all'estremità destra della barra) ferma ogni cart in
  riproduzione, su tutte le pagine. La sua etichetta mostra quanti sono in
  riproduzione, come in **Ferma tutto (2)**; se non ce ne sono è attenuato e
  non mostra alcun numero.
- Mentre un cart suona, il suo bordo e una barra che cresce da sinistra
  durante la riproduzione prendono il [colore](#setting-up-carts) del cart
  (rosso per un cart senza colore), una linea sottile segna il bordo già
  riprodotto sopra la forma d'onda e il suo tempo scorre alla rovescia. La
  linea si muove in modo fluido, anche su un cart breve.
- Dopo che il file di un cart è stato analizzato, la sua forma d'onda compare
  in trasparenza dietro il nome, così distingui un suono che attacca subito
  da uno che entra in dissolvenza. Copre ciò che il cart riproduce, dal suo
  cue-in al suo cue-out.
- I cart per impostazione predefinita si **sovrappongono**: lanciarne un
  secondo non ferma il primo. Un cart impostato su **Ferma gli altri cart al
  lancio** ferma prima tutti gli altri cart in onda, su qualsiasi pagina.
- Un cart impostato su **In loop** riparte dal suo cue-in quando arriva alla
  fine, senza pause, finché non lo fermi.
- **Fai clic destro** su un cart per altre opzioni:

| Voce | Azione |
|---|---|
| Preascolta nel CUE | Lo riproduce sull'uscita CUE della cartwall (attenuata quando la cartwall non ha un'uscita Cue). Sull'uscita in onda chiede prima se lì suona qualcosa (vedi [CUE](players.md#cue-pre-listen)) |
| Ferma | Lo ferma |
| Modifica… | Lo apre nelle Impostazioni |

Il menu di un cart vuoto ha solo **Modifica…**, per scegliere il suo file.

- **Pagine:** le schede accanto a **CARTWALL** cambiano pagina. Un punto
  rosso indica che un cart di quella pagina è in riproduzione.
- Fai clic su **CARTWALL** per comprimere la cartwall o espanderla di nuovo.
  Compressa accanto ai lettori è una striscia stretta con una freccia per
  espanderla e, mentre un cart suona, un punto rosso.
- Sotto i lettori, alta quanto servono le sue righe, i pulsanti si
  rimpiccioliscono quando la finestra è bassa (fino a un'altezza minima) in
  modo che entrino tutte le righe configurate. Dopo che hai ridimensionato la
  cartwall, o accanto ai lettori, i pulsanti si allargano per riempirla, e i
  nomi e i tempi mantengono la loro dimensione al centro di ogni pulsante. In
  entrambi i casi la cartwall scorre solo quando nemmeno i pulsanti più
  piccoli entrano.

| Aspetto del pulsante | Significato |
|---|---|
| Punto viola | Jingle |
| Punto ambra | Effetto |
| Punto grigio | Spot (pubblicitario) |
| Bordo, tinta e punto colorati | Il colore del cart, scelto nelle Impostazioni: il punto prende il colore al posto di quello del tipo |
| ↻ dopo il tipo | È in loop |
| ✋ dopo il tipo | Ferma gli altri cart al lancio |
| File con una croce / segno di avviso | Il file manca / non si può decodificare; passa il puntatore sul cart per vedere il motivo e il percorso. Un file mancante viene cercato di nuovo ogni 30 s (`tuning.missing_recheck_ms`). |
| «Vuoto», attenuato | Nessun file assegnato |

I cart usano gli stessi marker dei brani. Partono dal loro cue-in e finiscono
al loro cue-out, che puoi modificare sulla forma d'onda di un lettore quando
il file è caricato lì.

## Tastiera {#keyboard}

Per impostazione predefinita **F1**…**F12** lanciano i cart 1–12 della pagina
mostrata, e **Ctrl+Space** ferma tutti i cart (come **Ferma tutto**; ferma
anche un cart che stai preascoltando nel CUE, anche quando nessun cart è in
riproduzione). Vedi [Tastiera](keyboard.md) per cambiarle.

Il tasto che comprime o espande la cartwall non fa nulla mentre la cartwall è
nascosta dalla barra superiore (vedi
[Layout](layout.md#show-and-hide-panels)).

## Configurare i cart {#setting-up-carts}

Vai a **Impostazioni → Cartwall**:

- **Pagine:** crea, rinomina, elimina (l'ultima pagina non si può eliminare)
  e imposta la dimensione della griglia (righe × colonne). Una griglia più
  piccola viene rifiutata se eliminerebbe dei cart che hanno un file. Una
  dimensione digitata nella casella accanto a un cursore si applica quando
  premi Invio o esci dalla casella; Esc la annulla.
- **Importa… / Esporta…** salvano una pagina in un file `.cartpage.json` e la
  ricaricano, per esempio per condividerla tra studi. I percorsi relativi
  dei file vengono risolti rispetto alla cartella del file. I colori viaggiano
  con la pagina; un file di una versione precedente si importa senza.
- **Cart:** fai clic su un cart nella griglia, poi imposta il suo nome, il
  file (**Scegli…** o **Svuota**), il tipo, **Colore**, **In loop** e **Ferma
  gli altri cart al lancio**. **Colore** offre nessun colore e otto colori
  (rosso, arancione, ambra, verde, turchese, blu, viola, rosa): dai a tutti
  gli spot un colore e agli identificativi della stazione un altro per
  trovarli a colpo d'occhio. Un cart vuoto conserva il suo colore, e
  svuotare il file di un cart non lo rimuove. Un cart con un file ha anche
  **Escludi dalla normalizzazione** e **Trim**, le impostazioni di
  [normalizzazione del loudness](loudness.md#per-track-exclude-and-trim) del
  suo brano, con la misura sotto l'etichetta.

Le uscite Main e Cue proprie della cartwall si trovano in **Impostazioni →
Uscite audio** (la riga **Cartwall**).
