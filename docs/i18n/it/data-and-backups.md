# Dati e backup

## Dove si trovano i file {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Impostazioni (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Playlist e sessione | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Cache di analisi | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Log e report dei crash | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

I percorsi Linux seguono le variabili XDG (`XDG_CONFIG_HOME` e così via)
quando sono impostate.

## Modalità portatile {#portable-mode}

Imposta la variabile d'ambiente `FAUSTE_HOME` su una cartella, e tutto viene
conservato lì, in `config/`, `data/`, `cache/` e `logs/`. È utile su una
chiavetta USB, o per tenere affiancate configurazioni separate.

## File {#files}

| File | Contenuto |
|---|---|
| `config.json` | Impostazioni, uscite, soglie di analisi, regolazioni avanzate |
| `playlists.json` | Playlist, brani, segni di riprodotto, marker manuali |
| `session.json` | Per ogni lettore: playlist mostrata, brani attuale e successivo, modalità, posizione, volume, larghezze delle colonne (per colonna) |

## Salvataggio automatico e backup {#autosave-and-backups}

- Le modifiche vengono salvate circa un secondo dopo che avvengono. Mentre
  qualcosa suona, la sessione (le posizioni) viene aggiornata allo stesso
  ritmo.
- Ogni salvataggio scrive un file temporaneo e poi sostituisce quello
  vecchio, quindi un'interruzione di corrente non lascia mai un file scritto
  a metà.
- Le versioni precedenti vengono conservate come `.bak1`, `.bak2` e `.bak3`.
- Se un file non si può leggere, viene usato il backup valido più recente. Il
  file illeggibile viene conservato, rinominato `*.corrupt-<time>`, per
  essere esaminato. L'applicazione si avvia sempre.

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
