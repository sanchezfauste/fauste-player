# Impostazioni

Apri **Impostazioni** nella barra superiore. Chiudile con **Chiudi** o con
`Esc`. La maggior parte delle modifiche si applica subito e viene salvata
automaticamente.

La finestra ha un'unica dimensione (900 × 640, più piccola su uno schermo
piccolo) qualunque sia la sezione, e la sezione scorre al suo interno. Ogni
sezione allinea le sue etichette in un'unica colonna.

Le sezioni Lettori, Meter, Analisi e Scorciatoie da tastiera hanno nella
loro intestazione un pulsante **Ripristina i valori predefiniti**. Chiede
conferma e poi reimposta soltanto quella sezione (Lettori mantiene il numero
di lettori e la lingua; Scorciatoie non ha altri pulsanti di ripristino).
Uscite audio, Playlist, Cartwall, MIDI e Remoto non lo hanno.

## Riavvio in sospeso {#restart-pending}

Alcune modifiche hanno effetto solo quando l'applicazione riparte: il
sistema audio, la frequenza di campionamento, la dimensione del buffer
(anche quella propria di un dispositivo), le uscite Main e Cue (lettori e
cartwall), i dispositivi bit-perfect e le impostazioni DSD. Il numero di
lettori non è tra queste: si applica subito.

Una frequenza o un buffer assegnati a un dispositivo contano solo quando
cambiano con cosa il dispositivo si apre: assegnare a un dispositivo lo
stesso valore di quello globale, o cancellare un tale valore, non è in
sospeso.

I limiti e le regolazioni del motore si applicano anch'essi al prossimo
avvio, ma si modificano nel file di configurazione con l'applicazione
chiusa (vedi [Dati e backup](data-and-backups.md)), quindi non risultano mai
in sospeso.

Mentre una di queste è in attesa, il piè di pagina delle Impostazioni dice
«Alcune modifiche hanno effetto dopo un riavvio.» e offre **Riavvia ora**, e
la barra superiore mostra una pillola **Riavvio in sospeso**. Passa il
puntatore sulla pillola per vedere cosa è in attesa. Un breve avviso (per
esempio, che un'impostazione è stata salvata) può prendere per un momento il
posto del testo del piè di pagina; **Riavvia ora** rimane. Entrambi fanno la
stessa cosa:

- Quando nulla è in onda, **Riavvia ora** (o la pillola) riavvia subito.
- Quando qualcosa è in onda, compare la finestra che elenca ciò che sta
  suonando, con **Ferma e riavvia** o **Annulla**.

La sessione viene salvata per prima e l'audio e il controllo MIDI si
fermano, poi l'applicazione riparte con la stessa cartella dei dati
(`FAUSTE_HOME`), e dopo nulla va in onda da solo. Se l'applicazione non
riesce a ripartire (in un Flatpak, anche quando la nuova non si avvia in
tempo), lo dice; avviala dal menu delle applicazioni.

## Uscite audio {#audio-outputs}

Le modifiche in questa sezione aspettano un riavvio: vedi
[Riavvio in sospeso](#restart-pending).

![Impostazioni, Uscite audio, vista Base: il selettore, il sistema audio, la frequenza di campionamento, la dimensione del buffer e le uscite Main e Cue di ogni lettore (qui il sistema silenzioso)](../../images/guide/settings-outputs.png)

In alto, **Mostra** sceglie **Base** o **Avanzate**. Base mostra il sistema
audio, la frequenza di campionamento, la dimensione del buffer e le uscite.
Avanzate aggiunge, per ogni dispositivo che un'uscita usa, la sua frequenza e
il suo buffer, l'interruttore bit-perfect e la modalità DSD, e poi le
impostazioni DSD. Cambiare vista mostra o nasconde soltanto delle righe:
nulla viene modificato o reimpostato. Quando Base nasconde
un'impostazione in uso, una riga lo dice. Quando nessuna uscita usa più un
dispositivo, la sua frequenza e il suo buffer, il suo interruttore
bit-perfect e la sua modalità DSD vengono dimenticati alla prossima
partenza dell'applicazione: se dopo di allora un'uscita lo usa di nuovo,
parte dai valori globali. Fino ad allora, sceglierlo di nuovo (per esempio
dopo aver scambiato due dispositivi) li mantiene.

| Impostazione | Significato |
|---|---|
| Sistema audio | L'ultima scelta, **Nessuna uscita (silenzio)**, non riproduce nulla: le linee temporali scorrono a passo reale senza scheda audio (per una macchina che non ne ha una, o per fare prove). Linux: PipeWire (nelle build che lo includono), PulseAudio, JACK o ALSA. Windows: WASAPI, ASIO (nelle build che lo includono) o JACK. macOS: Core Audio o JACK. I sistemi assenti su questo computer, o senza dispositivo di uscita (un server JACK non in esecuzione), sono mostrati come non disponibili. «Predefinito di sistema» usa il primo disponibile in quell'ordine. |
| Frequenza di campionamento | La frequenza a cui funziona ogni uscita, a meno che un dispositivo non ne abbia una propria (Avanzate); i file vengono convertiti a essa con un ricampionamento di alta qualità. I dispositivi bit-perfect partono alla loro frequenza e poi seguono i file. |
| Dimensione del buffer | Frame per blocco audio, a meno che un dispositivo non ne abbia uno proprio; la latenza risultante è mostrata sotto |
| Uscite per lettore | Per ogni lettore, un dispositivo **Main** (in onda) e un dispositivo **Cue** (preascolto), ciascuno con una coppia di canali. Una scheda audio che offre più profili di uscita (ALSA elenca anteriore, surround, hardware diretto…) mostra ciascuno come *scheda — profilo*; due voci che altrimenti risulterebbero uguali ricevono tra parentesi l'id del dispositivo. Le interfacce multicanale possono portare più lettori su coppie diverse. |
| Prova Main / Prova Cue | Riproduce un breve tono (1 kHz su Main, 440 Hz su Cue, 1,5 s, −18 dBFS) sull'uscita scelta, così puoi controllare i collegamenti prima di andare in onda |
| Cartwall | Le uscite Main e Cue della cartwall. Main usa per impostazione predefinita l'uscita di sistema. Senza un Cue non c'è il preascolto dei cart. |
| Frequenza di campionamento: *dispositivo* (Avanzate) | **Globale (...)** usa la frequenza di campionamento qui sopra; un valore dà a questo dispositivo una frequenza propria. Vengono offerte solo le frequenze che il dispositivo indica; una frequenza salvata che non indica più resta in elenco con una nota che potrebbe non aprirsi (il dispositivo ripiega allora sulla frequenza globale). I valori propri si applicano solo ai dispositivi che un'uscita nomina, non all'uscita predefinita di sistema, a meno che una non la nomini. |
| Dimensione del buffer: *dispositivo* (Avanzate) | **Globale (...)** usa la dimensione del buffer qui sopra; un valore dà a questo dispositivo una dimensione propria, con la sua latenza sotto. Un dispositivo che non accetta una dimensione propria del buffer ripiega su quella globale, e anche sulla frequenza globale quando non accetta nemmeno una frequenza propria. |
| Bit-perfect: *dispositivo* (Avanzate) | Un dispositivo bit-perfect viene aperto con accesso esclusivo e segue la frequenza di campionamento di ogni file finché non vi suona nulla. L'interruttore è disattivato dove il dispositivo non può dare l'accesso esclusivo. Vedi [Uscita bit-perfect](bit-perfect.md). |
| DSD: *dispositivo* (Avanzate) | **Converti in PCM** (il valore predefinito), **DoP** o, su Linux, **DSD nativo**. Ogni dispositivo lo mostra; vengono offerte solo le modalità che il dispositivo può accettare, e una riga sotto dice perché le altre no. Vedi [DSD](bit-perfect.md#dsd). |
| Quando un'altra sorgente ha bisogno di un'uscita DSD (Avanzate) | **Prosegui il brano DSD in PCM** (il valore predefinito), oppure **Mantieni il DSD e silenzia le altre sorgenti**. Vedi [DSD](bit-perfect.md#dsd). |
| Silenzio DSD (Avanzate) | Silenzio inviato prima che inizi un flusso DSD, dopo la sua fine e al passaggio a PCM, perché il convertitore si agganci senza clic; 200 ms per impostazione predefinita, da 0 a 2000. |

Un'uscita Cue non ripiega mai sull'uscita che usa Main, in modo che il
preascolto non vada mai in onda. Un Cue che nomina un dispositivo di un
sistema audio che questo computer non ha, o la stessa uscita (dispositivo e
canali) di Main, significa «nessun cue». Quando un'uscita Cue coincide con la
sua uscita Main, un avviso sotto di essa lo segnala. Un lettore senza
un'uscita Cue, o con il suo Cue sulla sua uscita Main, ha il pulsante **CUE**
attenuato; passandoci sopra il puntatore ti dice di scegliere qui
un'uscita Cue. Lo stesso vale per **Preascolta nel CUE** della cartwall.

Se un dispositivo scompare durante la riproduzione, i lettori mantengono le
loro linee temporali, e il dispositivo viene riaperto quando torna (vedi
[Risoluzione dei problemi](troubleshooting.md)).

## Lettori {#players}

![Impostazioni, Lettori: numero di lettori, modalità predefinita, durata della dissolvenza, mix automatico, cue-in e cue-out, avviso di fine brano e lingua](../../images/guide/settings-players.png)

| Impostazione | Predefinito | Significato |
|---|---|---|
| Lingua | Sistema | Lingua dell'interfaccia |
| Numero di lettori | 4 | Colonne nella schermata principale (un lettore in onda non si può rimuovere) |
| Modalità predefinita | CONT | La modalità con cui partono i lettori |
| Durata della dissolvenza | 1000 ms | Usata da Play quando è in onda e da Stop in dissolvenza |
| Mix automatico al punto MIX | Attivo | Sovrappone i brani in modalità continua |
| Usa cue-in e cue-out | Attivo | Disattivato: i lettori suonano ogni brano dall'inizio alla fine del file; i marker vengono mantenuti e i cart usano comunque i propri. Le durate e i totali delle playlist seguono lo stesso intervallo |
| Avviso di fine brano | 10 s | Quando il conto alla rovescia inizia a lampeggiare in rosso |

## Meter {#meters}

![Impostazioni, Meter, con il meter di picco digitale scelto](../../images/guide/settings-meters.png)

Le modifiche si applicano subito. Le Impostazioni mostrano solo ciò che usa
il tipo di meter scelto: i meter EBU, DIN e VU hanno la scala, la zona rossa
e il comportamento che il loro standard stabilisce (si imposta solo il
livello di allineamento), e l'allineamento di un meter K-System è il suo
0. Un valore che imposti viene mantenuto per quando sceglierai di nuovo
quel tipo.

| Impostazione | Predefinito | Significato |
|---|---|---|
| Tipo di meter | Picco digitale | Come sale e scende la barra, e la sua scala, secondo uno standard (vedi sotto) |
| Tempo di salita, Velocità di discesa | 5 ms, 11,8 dB/s | Solo per **Personalizzato**. Il tempo di salita è un tempo di integrazione: un burst di tono lungo quanto lui si legge 2 dB più basso; 0 mostra ogni picco. |
| True peak | Disattivato | Solo per picco digitale, personalizzato e K-System. Misura tra i campioni, con il filtro di sovracampionamento 4× pubblicato da ITU-R BS.1770. Mostra i picchi che superano 0 dBFS dopo la conversione, che un meter di picco di campione non vede. Come consente lo standard, un click isolato di un solo campione può leggersi fino a circa 0,3 dB sotto il suo valore di campione. |
| Fondo scala | −60 dBFS | Il fondo della scala digitale (picco digitale e personalizzato). Gli altri meter mostrano l'intervallo che dà il loro standard. |
| Tenuta del picco | 2 s | Solo per picco digitale, personalizzato e K-System: per quanto tempo resta acceso il livello più alto; 0 la disattiva. I meter di programma e il VU non hanno tenuta. |
| Livello di allineamento | −18 dBFS | Tutti tranne il K-System. Segnato sulla scala (EBU R68). È anche dove si trovano il segno EBU TEST, il segno DIN −9 e lo 0 VU. |
| Avviso da | −9 dBFS | Giallo da qui (massimo consentito EBU), per i meter di picco digitale e personalizzato |
| Pericolo da | −3 dBFS | Rosso da qui, per i meter di picco digitale e personalizzato. Gli altri diventano rossi dove lo fa la loro scala: VU da 0 VU, PPM EBU e DIN dal massimo consentito (EBU +9, DIN 0), il K-System da +4. |
| Lettura della loudness | Breve termine | La loudness sotto il meter: disattivata, momentanea (ultimi 400 ms) o a breve termine (ultimi 3 s), EBU R128 |
| Loudness di riferimento | −23 LUFS | La lettura è verde entro ±1 LU (EBU R128) |

| Tipo di meter | Standard | Comportamento |
|---|---|---|
| Picco digitale | IEC 60268-18 | Mostra ogni picco subito; scende di 20 dB in 1,7 s |
| PPM EBU | IEC 60268-10 tipo IIb | I picchi più brevi di circa 10 ms si leggono più bassi (un burst di tono di 10 ms si legge circa 1,6 dB più basso, uno di 0,5 ms circa 18 dB più basso), entro le tolleranze della EBU Tech 3205; scende di 24 dB in 2,8 s |
| PPM DIN | IEC 60268-10 tipo I | Lo stesso con un tempo di integrazione di 5 ms; scende di 20 dB in 1,5 s |
| VU | IEC 60268-17 | Il livello medio, con il movimento dell'ago di un VU meter: 99 % in 300 ms, con un leggero overshoot; una sinusoide legge il suo livello di picco |
| K-20, K-14, K-12 | K-System | Due sezioni: la media (RMS, 600 ms) come barra piena e il picco (scende di 26 dB in 3 s) attenuato sopra di essa. 0 è 20, 14 o 12 dB sotto il fondo scala; verde sotto 0, ambra da 0 a +4, rosso sopra. K-12 si adatta alla radiodiffusione, K-14 e K-20 ai programmi più dinamici. |
| Personalizzato | — | Il tuo tempo di salita e la tua velocità di discesa |

Ogni meter usa la scala del suo standard, con i suoi segni tra i canali:

| Meter | Scala |
|---|---|
| Picco digitale, personalizzato | −60 … 0 dBFS, segni ogni 10 dB fino a −40 e ogni 5 dB sopra; i 20 dB superiori occupano metà dell'altezza |
| PPM EBU | −12 … +12 attorno al livello di allineamento (TEST), ogni 4 dB; i livelli più bassi restano in fondo |
| PPM DIN | −50 … +5, dove 0 è 9 dB sopra il livello di allineamento (−9 dBFS per impostazione predefinita) |
| VU | −20 … +3 VU, 0 VU al livello di allineamento; la barra si muove in proporzione alla tensione, come l'ago |
| K-System | da +20, +14 o +12 (0 dBFS) fino a −60; uniforme in dB fino a −24 |

## Analisi {#analysis}

![Impostazioni, Analisi: le soglie dei marker automatici](../../images/guide/settings-analysis.png)

Le soglie descritte in [Marker e mix](markers-and-mixing.md).
**Rianalizza tutti i brani** esegue di nuovo l'analisi per l'intera
libreria; i marker manuali vengono mantenuti.

Dopo un aggiornamento la cui analisi è cambiata, i brani analizzati dalla
versione precedente mantengono i loro marker e le loro forme d'onda, che
funzionano ancora. All'avvio Fauste Player dice quanti sono e offre
**Analizza ora** o **Più tardi**; **Analizza i brani obsoleti (N)** qui fa lo
stesso in qualsiasi momento. I brani sui lettori vengono comunque aggiornati,
man mano che vengono mostrati, e così i brani dei cart che non hanno un
formato registrato (un cart suona bit-perfect solo quando il suo formato è
noto). I brani il cui file manca non vengono contati finché il file non
torna.

## Playlist {#playlists}

![Impostazioni, Playlist: la cartella della musica, le playlist e le colonne della tabella](../../images/guide/settings-playlists.png)

- **Cartella della musica:** da dove partono le finestre di dialogo dei file.
- **Nuova playlist**, **rinomina** (modifica il nome e premi Invio; Esc
  annulla) ed **elimina** (icona del cestino).
- **Importa M3U / PLS…** crea una nuova playlist da un file di playlist.
  **M3U** su ogni riga la esporta come M3U8. Vedi [Playlist](playlists.md).
- **Colonne della tabella:** quali colonne mostrano le tabelle dei brani e in
  quale ordine, per ogni lettore: una casella per colonna (Titolo e Durata
  non si possono disattivare), frecce su e giù per quelle visibili, e
  **Colonne predefinite**. Vedi [Playlist](playlists.md).

**Lingua:** un elenco a discesa: **Sistema** (segue il sistema operativo),
poi ogni lingua in cui è disponibile l'interfaccia, ciascuna con il proprio
nome (prima l'inglese, poi in ordine alfabetico: per esempio Español).
L'interfaccia cambia subito. Una lingua di sistema senza una traduzione
propria usa quella più vicina (il francese canadese usa il francese, il
portoghese brasiliano usa il portoghese), e altrimenti l'inglese. Una lingua
nel file delle impostazioni che l'interfaccia non ha viene mostrata come
**Sistema** e segue il sistema operativo.

Inglese e spagnolo sono scritti a mano. Le altre traduzioni sono state
generate con l'IA e possono contenere errori; quando se ne usa una,
**Informazioni su Fauste Player** lo dice. Le correzioni di madrelingua sono
benvenute come issue o pull request.

## Cartwall {#cartwall}

![Impostazioni, Cartwall: le pagine, la griglia e l'editor del cart selezionato](../../images/guide/settings-cartwall.png)

Pagine, dimensione della griglia, l'editor dei cart, e importazione ed
esportazione delle pagine di cart. Vedi [Cartwall](cartwall.md).

## Scorciatoie da tastiera {#keyboard-shortcuts}

![Impostazioni, Scorciatoie da tastiera: ogni azione dei lettori con il suo tasto, e Rimuovi accanto a quelle assegnate](../../images/guide/settings-shortcuts.png)

Vedi [Tastiera](keyboard.md).

## MIDI {#midi}

![Impostazioni, MIDI, con il controllo MIDI disattivato](../../images/guide/settings-midi.png)

Attiva le superfici di controllo MIDI, vedi le porte di ingresso e fai
apprendere un controllo per ogni azione dei lettori. Vedi
[Superfici di controllo MIDI](midi.md).

## Remoto {#remote}

![Impostazioni, Remoto, con l'API HTTP in ascolto su questo computer](../../images/guide/settings-remote.png)

Controllo remoto in rete, per pagine web, app per telefono, automazione e
superfici di controllo. Vedi [Controllo remoto](remote-control.md).

- **Consenti il controllo remoto via HTTP**, il suo **indirizzo** e la sua
  **porta**, e una riga che dice se è in ascolto.
- **Token**, richiesto oltre questo computer. **Genera** ne crea uno
  casuale, **Mostra** lo rivela e **Copia** lo mette negli appunti. Compare
  un avviso quando l'indirizzo arriva oltre questo computer e non c'è un
  token.
- **Pagine web autorizzate a usare l'API**: un'origine per riga.
- **Consenti il controllo via OSC**, il suo **indirizzo** e la sua
  **porta**, e i **mittenti autorizzati** (indirizzi o sottoreti, uno per
  riga).
- **Pubblica i tempi ogni**: ogni quanto vengono inviati il tempo trascorso
  e quello rimanente mentre qualcosa suona.

I campi di testo e i numeri si applicano quando li lasci, il che include
aprire un'altra sezione o chiudere le Impostazioni; Esc annulla ciò che
stavi digitando. Un valore non valido viene corretto, e il campo mostra ciò
che è stato mantenuto.
