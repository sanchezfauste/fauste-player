# Reproductors

Cada columna és un reproductor. Els reproductors són independents: cadascun
té les seves pròpies pestanyes de llistes, transport, volum i sortides.

![El reproductor 1 en antena: capçalera, caràtula, títol, pista següent, transport, compte enrere, vúmetre, fader i forma d'ona](../../images/guide/player.png)

## Capçalera {#header}

| Element | Significat |
|---|---|
| `P1` … `Pn` | Número del reproductor (la tecla numèrica que el reprodueix) |
| Punt d'estat i etiqueta | **En antena** (vermell), **En pausa** (ambre), **Aturat** (gris) |
| Distintiu **Mescla** / **Fos** | Hi ha en marxa un fos encadenat cap a la pista següent, o un stop amb fos |
| Distintiu **Stop al final** | El reproductor s'atura quan acaba la pista actual |
| Distintiu **Repetir** / **Aturar després** | La pista actual es repeteix, o atura el reproductor quan acaba, per la seva pròpia marca al menú de la llista. Posa-hi el ratolí a sobre per veure la frase sencera. El botó **Stop al final** del reproductor té prioritat: mentre és activat, només se'n mostra el distintiu |
| **BP** / **DSD** | **BP** és encès mentre la pista actual arriba sense canvis al seu dispositiu Main. **DSD** el substitueix mentre una pista DSD surt com a DSD, sense canvis (vegeu [Sortida bit perfect](bit-perfect.md)). **Altres silenciades** apareix al costat quan aquest flux DSD manté altres fonts fora de la sortida |
| **SINGLE** \| **CONT** | Mode de reproducció (vegeu més avall): un sol control unit, la meitat encesa és el mode actiu |
| **CUE** | Preescolta de la pista següent a la sortida CUE |

## Fila d'informació {#info-row}

- **Caràtula** de la pista en antena, o una imatge de disc de vinil si no n'hi
  ha.
- **Títol i artista** de la pista en antena. Un reproductor aturat mostra la
  pista que iniciarà Play (la seva següent), amb la caràtula, la durada i la
  forma d'ona, a punt al cue-in o allà on hagis fet clic a la forma d'ona.
- Una pista reproduïda o carregada abans que acabi la seva anàlisi ja té la
  durada si la capçalera del fitxer la dona: el compte enrere,
  `transcorregut / total` i el clic per saltar funcionen de seguida, sobre una
  línia plana fins que la forma d'ona és a punt. Quan la capçalera no la
  desa (fitxers AAC sense format contenidor, fitxers MP3 sense marc de
  durada, Matroska i WebM), el total mostra «—» i no es pot fer clic a la
  forma d'ona fins que acaba l'anàlisi.
- **Vúmetre estèreo** (a la columna de la dreta del reproductor, al costat
  del fader; ocupa la fila d'informació i el transport): el nivell que
  treu el reproductor, després del seu volum.
  - Una barra contínua per canal, a l'escala de la norma del tipus de
    mesurador (per defecte el mesurador de pic digital: de −60 a 0 dBFS, amb
    els 20 dB superiors ocupant la meitat de l'alçada). L'escala és una regla
    a cada costat de les barres, amb etiquetes en les unitats pròpies del
    mesurador a tots dos costats. El màxim i el mínim (en el mesurador digital,
    el mínim de l'escala) sempre estan marcats; cada etiqueta té una marca a
    cada regla, i unes marques més curtes entre les etiquetes funcionen com
    les d'una regla de mesurar: repartides uniformement sobre valors rodons,
    cada 1 dB al mesurador EBU i cada 5 dB per sota de −20 al digital quan el
    mesurador és prou alt, i cada 2, 2,5, 5 o 10 dB (o cap) quan és massa baix
    per a elles. Al mesurador digital (i al personalitzat), un mesurador alt
    etiqueta més valors: cada 1 dB de −20 a 0, i cada 5 dB entre les marques
    de 10 dB per sota de −20 (−45, −55), sempre repartits uniformement i només
    allà on caben sense que les etiquetes es toquin; els altres tipus de
    mesurador conserven les etiquetes que dona la seva norma. El nivell
    d'alineació (−18 dBFS al mesurador digital) és una marca blanca més gruixuda
    a les dues regles. No es dibuixa res sobre les barres ni entre elles, de
    manera que el que veus a les barres és només el nivell, la retenció del pic
    i els colors.
  - La barra és verda, groga a partir del nivell d'avís (−9 dBFS) i vermella
    a partir del nivell de perill (−3 dBFS). Els altres tipus de mesurador
    es posen vermells allà on ho fa la seva escala (a partir de 0 VU, a
    partir del màxim permès en un PPM).
  - Els mesuradors K-System mostren dues seccions: la barra massissa és el
    nivell mitjà (RMS) i la part més fosca per sobre arriba fins al pic. Els
    seus colors són els del K-System: verd per sota de 0, ambre de 0 a +4,
    vermell per sobre.
  - Els mesuradors de pic digital, K-System i personalitzats mantenen
    il·luminat el nivell més alt durant un moment com una línia (la
    retenció del pic; la seva durada és **Retenció del pic** a
    [Configuració → Vúmetres](settings.md#meters), i 0 la desactiva). Els
    mesuradors PPM EBU, PPM DIN i VU no tenen retenció.
  - El número de dalt és el nivell més alt des que va començar l'entrada, en
    dBFS, vermell a la zona de perill. Es manté després d'un stop i torna a
    començar quan es reprodueix una entrada (la següent o la mateixa un altre
    cop), o quan hi fas clic.
  - El número de sota és la sonoritat en LUFS (EBU R128), verd a ±1 LU de
    l'objectiu (−23 LUFS).
  - El tipus de mesurador i cada nivell es poden canviar a
    [Configuració → Vúmetres](settings.md#meters).
  - **Lectures per sobre de 0 dBFS.** El mesurador mostra el que treu el
    reproductor, i això pot superar el fons d'escala. La barra s'atura a dalt
    de l'escala, de manera que el mateix vermell mostra 0 dBFS i qualsevol
    nivell per sobre; només el número de dalt diu fins on arriba, amb el seu
    signe (per exemple `+3.5`).
    - Un fitxer pot portar ell mateix nivells per sobre del fons d'escala (un
      fitxer de coma flotant, o un fitxer amb pèrdua els pics decodificats
      del qual el superen).
    - Convertir la freqüència pot crear pics entre les mostres: un senyal que
      toca 0 dBFS marca uns +3 dBFS després de passar de 44,1 a 48 kHz.
      L'opció de true peak també llegeix aquests pics.
    - El mesurador llegeix cada reproductor per separat, no la suma al
      dispositiu: dos reproductors en una sola sortida poden sumar-se per sobre
      del fons d'escala sense que cap mesurador ho mostri.
    - Res al reproductor afegeix guany per sobre del 100 %. Un dispositiu
      d'enters satura al fons d'escala; un dispositiu de coma flotant rep el
      nivell tal com és i el sistema de so o el controlador el satura.
- **Fader de volum** (a la dreta del mesurador, tan alt com ell): arrossega'l
  o usa la roda del ratolí, un pas per cada clic de la roda. La informació
  emergent mostra el nivell en dB; a dalt és 0 dB i a baix és silenci.
- **Títol, artista** i la línia **següent**, amb un quadrat verd. Mentre el
  CUE és actiu, la posició de preescolta es mostra en blau.

## Transport {#transport}

| Botó | Acció |
|---|---|
| **Play / Següent** (gran) | Aturat: inicia la pista següent. En antena: fos cap a la pista següent (el temps del fos es defineix a [Configuració](settings.md)). En pausa: reprèn. Amb la pista en antena com a següent, Play reinicia aquesta pista amb el fos habitual. |
| **Stop** | Atura a l'instant (amb una breu rampa antiespetec) |
| **Stop amb fos** | Fa un fos de sortida i atura |
| **Pausa** | Posa en pausa o reprèn; parpelleja en ambre mentre és en pausa |
| **Stop al final** (un triangle de reproducció i després un quadrat) | Atura quan acaba la pista actual, una sola vegada. En mode SINGLE només està disponible mentre la pista actual es repeteix: acaba la repetició quan acaba la passada que està sonant. Per aturar després d'una pista cada vegada que es reprodueix, o per repetir una pista, usa el seu menú a la llista (vegeu [Llistes de reproducció](playlists.md)) |
| **Pista anterior** (una barra i dos triangles) | En antena: fos de tornada cap a la pista que aquest reproductor va reproduir abans, com ho fa Següent. Prem de nou per continuar enrere. La pista que has deixat passa a ser la següent. |
| **Reiniciar la pista** (una barra i un triangle) | Torna a l'inici de la pista actual (el seu cue-in). Un reproductor en pausa continua en pausa. |

Els botons que ara no poden actuar apareixen atenuats: Stop i Reiniciar sense
res carregat, Pausa i Stop amb fos mentre està aturat, Anterior sense cap
pista prèvia o durant un fos. Un reproductor recorda les últimes 50 pistes que
ha reproduït (`players.history_len` a `config.json`, de 0 a 1000).

## Modes {#modes}

- **CONT (continu):** al punt MIX el reproductor inicia la pista següent i
  superposa el final de l'actual. Vegeu
  [Marcadors i mescla](markers-and-mixing.md).
- **SINGLE:** cada pista s'atura al final. *Stop al final* no està disponible
  en aquest mode, perquè cada pista ja s'atura, excepte mentre la pista
  actual es repeteix: aleshores acaba la repetició quan acaba la passada que
  està sonant.

## Compte enrere {#countdown}

El número gran és el temps que queda fins al final de la pista (el seu
cue-out), amb dècimes. El temps transcorregut i el total són a la fila de
sota la forma d'ona, a la dreta. Durant els últims segons abans del final (10
per defecte, es defineix a Configuració) el compte enrere parpelleja en
vermell.

## Forma d'ona {#waveform}

- La part ja reproduïda es dibuixa amb el color de la forma d'ona; la resta és
  més fosca.
- El contorn mostra els pics, tènuement; el cos massís de dins és el nivell
  mitjà (RMS). En una pista forta els pics omplen l'alçada, i el cos continua
  mostrant on la pista és més fluixa o més forta.
- Una àrea blava ombrejada a l'inici marca la **intro**, i un distintiu en fa
  el compte enrere. La intro només es mostra quan s'ha definit.
- Una àrea taronja ombrejada al final marca l'**outro**, amb el seu propi
  compte enrere.
- Una línia ambre discontínua amb una etiqueta **MIX** marca on comença la
  pista següent en mode continu. Apareix atenuada en mode single.
- Es dibuixa tot el fitxer. L'inici i el final en silenci que la reproducció
  omet (abans del cue-in i després del cue-out) es dibuixen més foscos, amb
  una línia fina allà on la reproducció comença i acaba. Amb **Usar cue-in i
  cue-out** desactivat (Configuració → Reproductors) res no és més fosc i
  les dues línies apareixen atenuades: la reproducció va de l'inici al final
  del fitxer, i el cue-in i el cue-out d'aquesta guia volen dir aquests dos
  extrems.
- Posa-hi el ratolí a sobre per veure el temps sota el punter. **Fes clic per
  saltar** allà. En un reproductor aturat, un clic tria on **Play** inicia la
  pista següent: el cap de reproducció i el compte enrere s'hi mouen, i no
  sona res fins que prems Play. Triar una altra pista com a següent, moure-la
  o treure-la, o Stop, torna al cue-in; també ho fa qualsevol altra manera
  d'iniciar una pista, i Reiniciar, Anterior i l'avanç automàtic sempre usen
  el cue-in. Un clic abans del cue-in (a l'inici més fosc) tria el cue-in. Un
  clic al cue-out o després (a la cua més fosca) cancel·la una tria anterior:
  Play comença al cue-in. Un clic és prémer i deixar anar sense moure el
  punter més d'uns quants píxels.
- **Prémer i arrossegar** mou la vista ampliada al llarg de la pista, com si
  l'agafessis. Un arrossegament mai no salta, i sense zoom no fa res.
  Alt+arrossegar continua editant els marcadors.
- **Roda del ratolí** sobre la forma d'ona: amplia i redueix al voltant del
  punter, fins al detall més fi que té l'anàlisi. **Maj+roda** (o una roda
  lateral) es mou al llarg de la pista. Mentre és ampliada, la vista segueix
  la posició de reproducció, excepte durant 10 segons després d'ampliar-la o
  moure-la (`ui.follow_current_grace_secs`; 0 desactiva el seguiment).
  **Vista completa**, a la cantonada superior dreta, reduir el zoom del tot o
  una pista nova tornen a mostrar la pista sencera.

## CUE (preescolta) {#cue-pre-listen}

Prémer **CUE** (o **Preescoltar al CUE** al menú d'una pista) reprodueix la
pista a la sortida CUE del reproductor, per exemple uns auriculars, sense
tocar la sortida en antena, i obre una petita **finestra de CUE** per a aquest
reproductor. Poden haver-hi diverses finestres obertes, una per reproductor.
Vegeu [Configuració](settings.md) per triar el dispositiu CUE. Un reproductor
necessita una sortida Cue que no sigui la seva sortida Main: sense una,
**CUE** i **Preescoltar al CUE** apareixen atenuats, i en posar-hi el ratolí
a sobre ho diuen.

![La finestra de CUE del reproductor 4, preescoltant la seva pista següent](../../images/guide/cue-window.png)

La finestra mostra:

- el títol i l'artista;
- la forma d'ona de tot el fitxer amb la posició del CUE. Funciona com la del
  reproductor: clic per saltar-hi, zoom amb la roda, arrossegar per moure's,
  **Vista completa**, els comptes enrere de la intro i l'outro, i els
  marcadors d'intro, outro i MIX, que s'editen aquí com al reproductor
  (vegeu [Marcadors i mescla](markers-and-mixing.md)). Un CUE reprodueix tot
  el fitxer, de manera que res no es dibuixa més fosc, el cue-in i el cue-out
  són línies atenuades, i l'outro compta enrere fins al final del fitxer. El
  zoom és propi: la forma d'ona del reproductor no es mou. Mentre el CUE
  sona, una vista ampliada en segueix la posició com la del reproductor; un
  CUE en pausa conserva la vista que has definit, de manera que pots ampliar
  i col·locar marcadors. Un CUE iniciat després de tancar la seva finestra, o
  en una altra pista, mostra el fitxer sencer;
- el temps transcorregut i el temps que queda fins al final del fitxer (un
  CUE reprodueix fitxers sencers);
- **Pausar el CUE** / **Reprendre el CUE**, **Aturar el CUE** i **Marcar com
  a següent**. **Marcar com a següent** fa que la pista en CUE sigui la
  següent del reproductor i manté el CUE sonant. Apareix atenuat quan la
  pista ja és la següent. Si la pista en CUE és la que és en antena, es
  reprodueix una vegada més quan acabi la passada actual.

Mentre el CUE és en pausa, el seu botó de pausa (que mostra **Reprendre el
CUE**) parpelleja en ambre, com el del propi reproductor.

Un salt en un CUE en pausa el manté en pausa. El botó de tancar de la
finestra, o **Aturar el CUE**, atura el CUE.

Mentre un CUE està en marxa, marcar una pista com a següent (doble clic) o
fer un sol clic en una fila mou el CUE a aquesta pista, des del seu cue-in;
si estava en pausa, es torna a reproduir. Una pista el fitxer de la qual
falta o no es pot llegir deixa el CUE on és.
