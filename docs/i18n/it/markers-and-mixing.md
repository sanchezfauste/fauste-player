# Marker e mix

Ogni brano ha fino a cinque **marker**, in secondi:

| Marker | Significato | Come viene impostato |
|---|---|---|
| Cue-in | Dove inizia la riproduzione | Automatico: poco prima del primo suono sopra la soglia di taglio |
| Cue-out | Dove finisce il brano | Automatico: poco dopo l'ultimo suono sopra la soglia di taglio |
| MIX (inizio del segue) | Dove inizia il brano successivo in modalità continua | Automatico (vedi sotto) |
| Inizio dell'outro | Dove comincia il finale del brano | Automatico (vedi sotto) |
| Fine dell'intro | Fine dell'introduzione sopra cui si parla | A mano, o da un tag `INTRO` nel file |

I marker impostati a mano hanno sempre la precedenza: una nuova analisi non
li sostituisce mai.

## Modificare i marker {#editing-markers}

Sulla forma d'onda di un lettore, o su quella della sua finestra CUE (lo
stesso menu e le stesse maniglie; una modifica si vede in entrambe
contemporaneamente):

- **Fai clic destro** dove vuoi un marker e scegli **Imposta qui il cue-in**,
  **Imposta qui la fine dell'intro**, **Imposta qui l'inizio dell'outro**,
  **Imposta qui il punto MIX** o **Imposta qui il cue-out**. **Ripristina i
  marker automatici** rimuove i marker che hai posizionato, e il brano viene
  analizzato di nuovo.
- **Tieni premuto Alt** (Option su macOS): sui marker compaiono delle
  maniglie. Trascinane una per spostarlo; il tempo viene mostrato mentre
  trascini. Un trascinamento non sposta mai la testina.

Il cue-in deve restare prima del cue-out. Gli altri marker restano tra i due.
Le modifiche al brano in onda si applicano subito alla sua prossima
transizione.

## Il tag INTRO {#the-intro-tag}

Un file può contenere il tempo della sua intro in un tag `INTRO`, in secondi
(`12.5`) o come `m:ss`. Il nome può essere scritto con qualsiasi
combinazione di maiuscole e minuscole (`INTRO`, `Intro`). Può essere un
frame di testo utente ID3v2 (MP3, WAV, AIFF, DSF), un commento Vorbis, Opus
o FLAC, un elemento APE (WavPack, Monkey's Audio) o un atomo libero MP4. Viene
letto durante l'analisi. Una fine dell'intro impostata a mano ha comunque la
precedenza.

## Come si trovano i marker automatici {#how-the-automatic-markers-are-found}

L'analisi misura i picchi del brano a passi di 10 ms e la sua loudness in
brevi finestre (50 ms per impostazione predefinita).

- **Cue-in / cue-out:** viene saltato solo il quasi-silenzio all'inizio e
  alla fine: tutto ciò il cui picco raggiunge la *soglia di taglio* (−60
  dBFS per impostazione predefinita), su uno dei due canali, viene mantenuto,
  con un *margine di taglio* (20 ms per impostazione predefinita) attorno.
  Dissolvenze in entrata morbide, code silenziose e suoni brevi non vengono
  mai tagliati.
- **MIX:** l'analisi trova l'ultimo punto in cui il brano è ancora a meno
  del *calo per il segue* (15 dB per impostazione predefinita) sotto la sua
  loudness tipica, così master forti e piani con la stessa dissolvenza si
  mixano allo stesso modo. Quel punto non è mai più di *durata massima del
  mix* (4 s per impostazione predefinita) prima del cue-out, così le
  sovrapposizioni restano brevi.
- **Outro:** l'analisi scorre all'indietro dal cue-out e trova dove il
  livello scende più del *calo di livello dell'outro* (6 dB per
  impostazione predefinita) sotto la loudness mediana del brano. L'outro non
  è mai più lungo di 30 s per impostazione predefinita.
- I brani più corti della *durata minima per i marker di mix e di outro* (60
  s per impostazione predefinita), come jingle e spot, non hanno MIX né
  outro.

### Registrazioni lunghe {#long-recordings}

Un intero programma (una, quattro o più ore) viene analizzato come un
brano, se serve mentre suona: un file FLAC o Opus di 4 ore richiede meno di
un minuto su un computer attuale, e la memoria non cresce con la durata. Lo
spostamento in un punto qualsiasi, anche vicino alla fine, è immediato. La
forma d'onda e i marker vengono mantenuti nella cache di analisi fino a
circa 16 ore di audio; un file più lungo funziona comunque, ma viene
analizzato di nuovo a ogni avvio di Fauste Player.

Tutti questi valori si trovano in **Impostazioni → Analisi**. Dopo averli
cambiati, i brani vengono analizzati di nuovo automaticamente.

## Cosa fa il lettore con i marker {#what-the-player-does-with-them}

- **Modalità continua con mix automatico attivo:** al punto MIX il brano
  successivo parte a pieno livello mentre quello attuale sfuma fino al suo
  cue-out. La sovrapposizione è precisa al campione.
- **Modalità continua senza un punto MIX,** o con il mix automatico
  disattivato: il brano successivo parte esattamente al cue-out, senza
  pause.
- **Modalità single**, o **Stop alla fine**: il lettore si ferma al cue-out.
- **Premere Play mentre è in onda:** il brano successivo parte subito e
  quello attuale sfuma nella *durata della dissolvenza* (1 s per
  impostazione predefinita).
- **Usa cue-in e cue-out** disattivato (Impostazioni → Lettori): ogni
  lettore suona ogni brano da 0 alla fine del file. Cue-in e cue-out,
  automatici e manuali, vengono mantenuti, e la forma d'onda li disegna come
  linee attenuate. Il punto MIX, l'intro e l'outro funzionano comunque,
  all'interno dell'intero file; **Mix automatico al punto MIX** è un
  interruttore separato. I conti alla rovescia, la colonna della durata, i
  totali delle playlist nelle Impostazioni e i tempi dell'API remota seguono
  lo stesso intervallo. I cart usano sempre il proprio cue-in e cue-out.
  Cambiare l'impostazione non riavvia, non sposta e non ferma mai un brano in
  riproduzione; il brano successivo viene preparato di nuovo.

Un brano può essere riprodotto prima che la sua analisi finisca. Fino ad
allora suona dall'inizio alla fine del file, senza punto MIX.
