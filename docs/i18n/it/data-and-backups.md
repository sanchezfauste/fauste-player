# Dati e backup

## Dove si trovano i file {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Impostazioni (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Libreria, playlist, cart e sessione (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Cache di analisi | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Log e report dei crash | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

I percorsi Linux seguono le variabili XDG (`XDG_CONFIG_HOME` e così via)
quando sono impostate.

Lo snap conserva invece i suoi file nella propria cartella: le impostazioni in
`~/snap/fauste-player/current/.config/fausteplayer/`, il database e i log in
`~/snap/fauste-player/current/.local/share/fausteplayer/`, e la cache di
analisi in `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` li elimina insieme allo snap (conserva per un
po' un'istantanea; con `--purge` non lo fa).

## Modalità portatile {#portable-mode}

Imposta la variabile d'ambiente `FAUSTE_HOME` su una cartella, e tutto viene
conservato lì, in `config/`, `data/`, `cache/` e `logs/`. È utile su una
chiavetta USB, o per tenere affiancate configurazioni separate.

## File {#files}

| File | Contenuto |
|---|---|
| `config.json` | Impostazioni, uscite, soglie di analisi, regolazioni avanzate |
| `fauste.db` | La libreria (brani, le loro categorie, marker manuali, tag, loudness), le playlist con i loro segni di riprodotto, le pagine di cart e la sessione: per ogni lettore la playlist mostrata, i brani attuale e successivo, la modalità, la posizione, il volume e le larghezze delle colonne (per colonna), e il layout della finestra |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Copie del database (vedi sotto) |
| `window.ron` | Dimensione e posizione della finestra principale e se era ingrandita (solo computer; su Wayland la posizione la sceglie il sistema) |

## Salvataggio automatico e backup {#autosave-and-backups}

- Le modifiche vengono salvate circa un secondo dopo che avvengono, e viene
  scritto solo ciò che è cambiato. Mentre qualcosa suona, la sessione (le
  posizioni) viene aggiornata allo stesso ritmo.
- Il database viene scritto in transazioni, quindi un'interruzione di
  corrente non lascia mai una modifica salvata a metà.
- Al primo salvataggio di ogni sessione, e poi circa ogni cinque minuti
  mentre le modifiche vengono salvate, l'applicazione copia il database in
  `fauste.db.bak1` e sposta più in alto le copie più vecchie, in `.bak2` e
  `.bak3`. Riavviare più volte di seguito spinge quindi fuori i backup più
  vecchi.
- Se il database non si può leggere, viene usato il backup valido più
  recente. Il file illeggibile viene conservato, rinominato
  `fauste.db.corrupt-<time>`, per essere esaminato. Se ogni copia fallisce,
  l'applicazione si avvia vuota. Si avvia sempre.
- Anche una copia inutilizzabile viene conservata: quando l'applicazione si
  avvia da un'altra copia (o da un database vuoto), quelle inutilizzabili
  vengono rinominate `fauste.db.bak1.corrupt-<time>` (o `.newer-<time>`),
  così il backup successivo non le spinge mai fuori.
- Se un'altra copia dell'applicazione sta usando il database, questa gira
  senza salvare e lo dice all'avvio.
- Un database scritto da una versione più recente dell'applicazione non
  viene aperto: viene conservato come `fauste.db.newer-<time>`, e
  l'applicazione si avvia dal backup più recente che riesce a leggere, o
  vuota se non ce n'è, così aprire i dati con una versione più vecchia non li
  distrugge mai.
- `config.json` viene salvato come prima: un file temporaneo sostituisce
  quello vecchio, e le versioni precedenti vengono conservate come `.bak1`,
  `.bak2` e `.bak3` accanto a esso.
- Se il database non si può usare affatto in una sessione (per esempio non si
  può rinominare perché la cartella è in sola lettura), l'applicazione gira
  senza salvare fino al prossimo avvio, e il log lo dice.

## Playlist e sessione di una versione precedente {#playlists-and-session-from-an-earlier-version}

Le versioni precedenti conservavano le playlist in `playlists.json`, le
pagine di cart in `carts.json` e la sessione in `session.json`. Il primo
avvio di questa versione li legge (con le loro copie `.bak`) una volta sola e
mette tutto in `fauste.db`: la libreria con i suoi marker, tag e loudness, le
playlist con i loro segni di riprodotto, le pagine di cart e la sessione.
Poi i vecchi file vengono rinominati con `.imported` in fondo
(`playlists.json.imported`, `playlists.json.bak1.imported`, e così via). Non
vengono più usati e puoi eliminarli quando sei sicuro che ci sia tutto.

Per tornare a una versione precedente:

1. Chiudi l'applicazione.
2. Sposta `fauste.db`, `fauste.db-wal`, `fauste.db-shm` e ogni
   `fauste.db.bak*` in una cartella fuori dalla cartella dei dati. Non
   eliminarli.
3. Togli `.imported` dai nomi dei vecchi file.
4. Avvia la versione precedente.

Tutto ciò che è stato salvato dopo l'aggiornamento resta solo nella cartella
in cui hai spostato i file: i vecchi file non lo hanno. Se aggiorni di nuovo
più tardi, i vecchi file vengono importati di nuovo, purché nella cartella
dei dati non ci sia alcun `fauste.db` o `fauste.db.bak*`.

Se un avvio viene interrotto durante l'importazione, il successivo la
completa. Se uno dei vecchi file non si può leggere (troppo grande, o scritto
da una versione più recente), l'applicazione non importa nulla e gira senza
salvare quella sessione, così i vecchi file restano dove sono; il log
nomina il file. Correggilo o rimuovilo e avvia di nuovo. Una volta che
`fauste.db` esiste, i vecchi file JSON non vengono più importati.

## Recupero dopo un crash {#crash-recovery}

Dopo un crash o un riavvio, ogni lettore ritorna con la sua playlist, i suoi
brani attuale e successivo e la sua posizione, ma **in pausa o fermo**. Nulla
va in onda da solo. Un lettore che era proprio alla fine del suo brano
ritorna invece all'inizio di quel brano, così premendo Play lo riproduce
invece di terminarlo subito.

## Modificare `config.json` a mano {#editing-configjson-by-hand}

Chiudi prima l'applicazione (se c'è audio in onda, chiede prima di chiudere).
I valori sconosciuti o fuori intervallo vengono corretti al valore valido più
vicino quando il file viene caricato, e le correzioni vengono registrate nel
log. Le sezioni `limits` e `tuning` contengono valori avanzati (limiti di
risorse, temporizzazioni del motore) che non sono nella finestra delle
Impostazioni.
