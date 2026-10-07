# Cartwall

La cartwall è la striscia di pulsanti sotto i lettori. Ogni pulsante, un
**cart**, riproduce all'istante un suono: jingle, effetti, spot. I cart
suonano sulle proprie uscite, indipendentemente dai lettori.

![La cartwall con un cart in riproduzione](../../images/guide/cartwall.png)

## Uso {#using-it}

- **Fai clic su un cart** per lanciarlo. **Fai di nuovo clic** per fermarlo.
- **Ferma tutto** (all'estremità destra della barra) ferma ogni cart in
  riproduzione, su tutte le pagine. La sua etichetta mostra quanti sono in
  riproduzione, come in **Ferma tutto (2)**; se non ce ne sono è attenuato e
  non mostra alcun numero.
- Mentre un cart suona, il suo bordo diventa rosso, una barra rossa si
  accorcia durante la riproduzione e il suo tempo scorre alla rovescia.
- I cart per impostazione predefinita si **sovrappongono**: lanciarne un
  secondo non ferma il primo. Un cart impostato su **Ferma gli altri cart al
  lancio** ferma prima tutti gli altri cart in onda, su qualsiasi pagina.
- Un cart impostato su **In loop** riparte dal suo cue-in quando arriva alla
  fine, senza pause, finché non lo fermi.
- **Fai clic destro** su un cart per altre opzioni:

| Voce | Azione |
|---|---|
| Preascolta nel CUE | Lo riproduce sull'uscita CUE della cartwall (attenuata quando la cartwall non ha un'uscita Cue distinta dalla sua Main) |
| Ferma | Lo ferma |
| Modifica… | Lo apre nelle Impostazioni |

Il menu di un cart vuoto ha solo **Modifica…**, per scegliere il suo file.

- **Pagine:** le schede accanto a **CARTWALL** cambiano pagina. Un punto
  rosso indica che un cart di quella pagina è in riproduzione.
- Fai clic su **CARTWALL** per comprimere la striscia o espanderla di nuovo.
- Quando la finestra è bassa, i pulsanti si rimpiccioliscono (fino a
  un'altezza minima) in modo che entrino tutte le righe configurate; la
  cartwall scorre solo quando nemmeno i pulsanti più piccoli entrano.

| Aspetto del pulsante | Significato |
|---|---|
| Punto viola | Jingle |
| Punto ambra | Effetto |
| Punto grigio | Spot (pubblicitario) |
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

## Configurare i cart {#setting-up-carts}

Vai a **Impostazioni → Cartwall**:

- **Pagine:** crea, rinomina, elimina (l'ultima pagina non si può eliminare)
  e imposta la dimensione della griglia (righe × colonne). Una griglia più
  piccola viene rifiutata se eliminerebbe dei cart che hanno un file.
- **Importa… / Esporta…** salvano una pagina in un file `.cartpage.json` e la
  ricaricano, per esempio per condividerla tra studi. I percorsi relativi
  dei file vengono risolti rispetto alla cartella del file.
- **Cart:** fai clic su un cart nella griglia, poi imposta il suo nome, il
  file (**Scegli…** o **Svuota**), il tipo, **In loop** e **Ferma gli altri
  cart al lancio**.

Le uscite Main e Cue proprie della cartwall si trovano in **Impostazioni →
Uscite audio** (la riga **Cartwall**).
