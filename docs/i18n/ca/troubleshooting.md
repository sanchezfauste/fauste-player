# Resolució de problemes

## Sense so {#no-sound}

1. Obre **Configuració → Sortides d'àudio** i prem **Provar Main** per al
   reproductor. Si sents el to, comprova el fader de volum del reproductor.
2. Si no sents res, tria un altre dispositiu o parell de canals. Els canvis a
   les sortides tenen efecte després d'un reinici: prem **Reiniciar ara** a
   Configuració.
3. A Linux, prefereix **PipeWire** o **PulseAudio** a Configuració → Sortides
   d'àudio → Sistema d'àudio. Comparteixen la targeta de so amb altres
   programes. **ALSA** parla directament amb la targeta i pot trobar-la
   ocupada.
4. **JACK** apareix com a no disponible («no output device») quan no hi ha cap
   servidor JACK en marxa. Inicia el servidor (per exemple amb QjackCtl) i
   reinicia l'aplicació. Posa el servidor JACK a la freqüència de mostreig de
   Configuració (48 kHz per defecte): JACK funciona a una sola freqüència per
   a tots els programes.
5. **PipeWire** no s'ofereix als arxius descarregables; arriben a PipeWire a
   través del seu servei PulseAudio, que funciona de la mateixa manera. Està
   disponible en les compilacions fetes amb la característica `pipewire`.

## Bit perfect {#bit-perfect}

- **El distintiu BP continua apagat.** Comprova cada condició a
  [Sortida bit perfect](bit-perfect.md#the-bp-badge): volum al 100 %, cap fos,
  res més a les mateixes sortides, un fitxer sense pèrdua que s'ha analitzat, i
  un dispositiu que funciona a la freqüència del fitxer.
- **Un breu silenci abans d'una pista.** El dispositiu bit perfect s'ha tornat
  a obrir a la freqüència de mostreig de la pista. Mantén la biblioteca a una
  sola freqüència per evitar-ho.
- **Una pista es reprodueix remostrejada, i el registre diu que el dispositiu
  és ocupat (Linux).** Per canviar la freqüència, l'aplicació tanca el
  dispositiu i el torna a obrir. En aquell moment el servidor de so (PipeWire)
  pot prendre la targeta. L'aplicació ho torna a provar unes quantes vegades;
  si la targeta continua ocupada, la pista es reprodueix a la freqüència
  actual del dispositiu, i la pista següent torna a demanar la seva
  freqüència. Per donar la targeta en exclusiva a l'aplicació, obre la
  configuració de so del sistema i posa el perfil d'aquella targeta a **Off**
  (o **Pro Audio**), de manera que el servidor de so deixi en pau el seu
  dispositiu `hw:`. El nombre d'intents i l'espera entre ells són
  `tuning.device_busy_retries` i `tuning.device_busy_retry_ms` al fitxer de
  configuració.
- **El dispositiu sona, però el distintiu BP continua apagat (Windows o
  macOS).** S'ha refusat l'accés exclusiu, i el dispositiu sona compartit.
  - Windows: potser un altre programa té el dispositiu en exclusiva, o el
    control exclusiu està desactivat a les propietats Avançat del dispositiu.
  - macOS: potser un altre programa té el dispositiu en mode hog, o el
    dispositiu ofereix les seves freqüències només com a interval continu (la
    majoria d'interfícies llisten freqüències fixes).
- **Un dispositiu `hw:` no es pot obrir (Linux).**
  - Potser un servidor de so té la targeta. Atura'l, o configura el servidor
    perquè deixi en pau aquella targeta, i reinicia l'aplicació.
  - Alguns DAC USB només accepten mostres de 24 bits empaquetades
    (`S24_3LE`), que la biblioteca d'àudio no admet. Usa aquella targeta a
    través de `plughw:` (no bit perfect) en lloc d'això.

### DSD {#dsd}

- **Una pista DSD es reprodueix convertida tot i que el dispositiu està en DoP
  o DSD natiu.** El registre en diu el motiu («DSD converted to PCM» i la
  causa) per a aquestes causes: el volum del reproductor no és al 100 %, hi
  sona alguna cosa més al dispositiu, més de dos canals, o un dispositiu que
  refusa la freqüència (DoP necessita la freqüència DSD dividida per 16, per
  exemple 176,4 kHz per a DSD64) o que no té format de 24 o 32 bits. Una pista
  que encara no s'ha analitzat es converteix en silenci, sense cap línia al
  registre: analitza-la (Configuració → Anàlisi) i torna-la a reproduir.
- **Només la primera pista d'un àlbum DSD surt com a DSD.** És l'ajust de
  mescla per defecte: les pistes que el reproductor inicia per si sol es
  reprodueixen convertides. Tria **Mantenir el DSD i silenciar les altres
  fonts** a Configuració → Sortides d'àudio per mantenir-les en DSD. Vegeu
  [DSD](bit-perfect.md#dsd).
- **La capçalera mostra DSD però el convertidor reprodueix soroll o no
  s'enganxa.** El convertidor no reconeix DoP (o el format natiu). Torna a
  posar el dispositiu a **Convertir a PCM**.
- **Un espetec quan una pista DSD comença, s'atura o deixa el DSD.** El
  convertidor necessita més silenci DSD: augmenta el **Silenci DSD** (200 ms
  per defecte) a Configuració → Sortides d'àudio, Avançat.
- **Altres reproductors o cartutxos són en silenci al dispositiu.** Hi sona una
  pista DSD amb **Mantenir el DSD i silenciar les altres fonts**; es mostra el
  distintiu **Altres silenciades**. Tornen a sonar quan acaba la pista.

## Avís de «Sortida perduda» {#output-lost-alert}

La barra d'estat mostra **Sortida perduda: &lt;dispositiu&gt;** quan un
dispositiu deixa de respondre. Els reproductors continuen comptant i mesclant
amb un rellotge intern, de manera que l'automatització no es bloqueja. El
dispositiu es torna a provar cada 2 segons i es reprèn quan torna. Torna a
connectar el cable o torna a encendre la interfície.

### «Sortida perduda» que mai no desapareix, amb una sortida `hw:` directa {#output-lost-that-never-clears-with-a-direct-hw-output}

Una targeta de so usada a través d'una sortida ALSA `hw:` directa (per exemple
una sortida bit perfect) la té només Fauste Player: el servidor de so
(PipeWire o PulseAudio) no la pot usar alhora. Si una altra sortida passa pel
dispositiu predeterminat del servidor de so i aquest dispositiu predeterminat
és la mateixa targeta, aquella sortida no s'inicia mai i es queda en **Sortida
perduda**. El registre diu «output device opened but never started» una
vegada.

Usa un sol camí per targeta: dirigeix totes les sortides d'aquella targeta pel
mateix dispositiu `hw:` (amb canals diferents si cal), o tria una altra
targeta com a sortida predeterminada del servidor de so a la configuració de
so del sistema.

## Una pista mostra una icona d'advertència o un fitxer amb una creu {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

El fitxer falta (mogut, esborrat, desmuntat: un fitxer amb una creu) o no es
pot decodificar (un senyal d'advertència). Els reproductors el salten. Posa-hi
el ratolí a sobre de la fila per veure quin dels dos és, i el camí del
fitxer.

Un fitxer que falta es torna a buscar cada 30 segons
(`tuning.missing_recheck_ms` al fitxer de configuració): quan es munta la
unitat o es torna a posar el fitxer, la pista es pot reproduir per si sola.
Un fitxer que no es pot decodificar es torna a comprovar per si sol amb el
mateix temporitzador, per mida i data de modificació: no es torna a decodificar
tret que un dels dos hagi canviat, per exemple quan s'acaba una còpia. Per
comprovar-lo a l'instant usa **Tornar a analitzar** al menú de la seva fila, o
**Configuració → Anàlisi → Tornar a analitzar totes les pistes** per a tota la
biblioteca.

## Talls d'àudio {#audio-dropouts}

La barra d'estat avisa durant 5 segons després de cada tall que l'aplicació
detecta: **P1: talls d'àudio (3)** quan la decodificació d'un reproductor no
ha seguit el disc (el recompte és de la pista que sona ara), i
**&lt;dispositiu&gt;: talls del dispositiu d'àudio (2)** quan el dispositiu de
sortida ha perdut un termini (un xrun). El registre també en deixa constància,
com a màxim una línia cada 10 segons per tipus, amb quants n'hi ha hagut. No
tots els sistemes d'àudio informen dels xruns (PulseAudio no; el mode
exclusiu de Windows no).


- Augmenta la **mida del búfer** a Configuració (i prem **Reiniciar ara**).
- A Linux, permet la planificació en temps real. L'aplicació la demana al
  sistema a través de rtkit (D-Bus). Ser membre del grup `audio` amb un límit
  `rtprio` també funciona.
- Evita les unitats de xarxa per a la música que sona en antena.

## «La interfície ha tingut un error» {#the-interface-hit-an-error}

S'ha detectat un error de dibuix. L'àudio no se n'ha ressentit. Prem
**Reiniciar la interfície**. Si us plau, informa'n amb els registres.

## Registres i informes de fallades {#logs-and-crash-reports}

Vegeu [Dades i còpies de seguretat](data-and-backups.md) per a la carpeta de
registres. Hi ha un fitxer de registre per dia, i es conserven els últims 14.
Els informes de fallades es desen com a `crash-<hora>.txt`. Defineix
`RUST_LOG=debug` a l'entorn per a més detall. Adjunta tots dos fitxers quan
informis d'un error.
