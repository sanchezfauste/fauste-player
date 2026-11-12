# Sortida bit perfect

Un dispositiu **bit perfect** rep les mostres de cada fitxer exactament tal
com són al fitxer: la mateixa freqüència de mostreig, els mateixos valors,
sense remostreig, canvi de volum ni mescla.

La sortida bit perfect és un **mode especial per a l'escolta audiòfila**:
escoltar un arxiu, alimentar una cadena hi-fi o de monitoratge, comprovar un
enllaç digital, allà on cal evitar qualsevol processament a l'ordinador. És
desactivat per defecte i no està pensat per a un programa musical mesclat.
Llegeix [Què costa](#what-it-costs) abans d'activar-lo.

## Què costa {#what-it-costs}

Per mantenir cada pista intacta, un reproductor la sortida Main del qual és en
un dispositiu bit perfect sona de manera diferent dels altres:

- **Sense fosos encadenats ni segues.** Cada pista es reprodueix fins al final
  (el seu cue-out, o el final del fitxer amb **Usar cue-in i cue-out**
  desactivat), i després comença la següent. **Mescla automàtica al punt MIX**
  i el punt MIX no s'usen: el marcador MIX es dibuixa atenuat i el distintiu
  **Mescla** no s'encén mai. **Play / Següent** i **Anterior** mentre una pista
  és en antena canvien a l'instant, amb una breu aturada sense espetec, en lloc
  de fer un fos.
- **Un breu silenci quan canvia la freqüència de mostreig.** Entre dues pistes
  les freqüències de mostreig de les quals difereixen, quan no sona res més al
  dispositiu, el dispositiu primer reprodueix el que encara té a la memòria
  intermèdia (el final de la pista), i després es torna a obrir a la freqüència
  nova, cosa que deixa un breu silenci (normalment unes quantes desenes de
  mil·lisegons, més llarg en alguns dispositius). Les pistes a la mateixa
  freqüència se segueixen sense buit ni superposició. Quan la freqüència no pot
  seguir (vegeu [Què passa en un dispositiu bit
  perfect](#what-happens-on-a-bit-perfect-device)), la pista següent comença a
  l'instant, convertida a la freqüència del dispositiu.
- **Sense normalització de la sonoritat.** Una pista en un dispositiu bit perfect
  sempre sona a guany unitari, fins i tot amb la [normalització de la
  sonoritat](loudness.md#bit-perfect-outputs) activada: la capçalera mostra
  **0.0 dB** en ambre, i la seva informació emergent en diu el motiu. Els altres
  reproductors i cartutxos es normalitzen com sempre.
- **El volum es manté al 100 %.** El fader de volum del reproductor és bloquejat
  a dalt: a la pantalla, en controladors [MIDI](midi.md) i per l'[API remota i
  l'OSC](remote-control.md). Passa al 100 % quan s'activa bit perfect per al
  dispositiu, i s'hi queda quan es desactiva.
- **El que continua funcionant:** **Stop amb fos** (fa un fos de sortida durant
  la durada de l'stop amb fos), **Pausa**, **Stop**, **Reiniciar**, les marques de
  repetició i de stop al final, els modes Single i Continu, el Cue (la
  preescolta, que no té fader i no es veu afectada), els cartutxos, la [sortida
  DSD](#dsd), i tots els reproductors la sortida Main dels quals és en un altre
  dispositiu, que mesclen com sempre. Un cartutx pot sonar al mateix
  dispositiu: mentre sona, la pista següent del reproductor comença a la
  freqüència actual del dispositiu, convertida (de manera que no és bit
  perfect), i no s'insereix cap silenci.
- **Quan usar-lo.** Activa'l quan cada pista hagi d'arribar al convertidor
  intacta. Deixa'l desactivat per a un programa en antena que necessita
  transicions mesclades o canvis de nivell. El mode es defineix per
  dispositiu, de manera que els reproductors mesclats poden quedar-se en un
  altre dispositiu.

## Posar un dispositiu en bit perfect {#setting-a-device-bit-perfect}

1. A **Configuració → Sortides d'àudio**, tria el dispositiu explícitament
   per a la sortida Main d'un reproductor (o de la cartutxera). Un reproductor
   que es deixa al predeterminat del sistema no pot ser bit perfect. Un
   dispositiu que cap sortida no utilitza ja perd el seu interruptor bit
   perfect i el seu mode DSD la propera vegada que s'inicia l'aplicació.
2. Tria **Avançat** a dalt de la secció. A **Ajustos per dispositiu**,
   activa **Bit perfect** al costat del dispositiu.
3. El dispositiu esdevé bit perfect tan bon punt no hi sona res (vegeu [Ajustos
   pendents](settings.md#settings-pending)).

El dispositiu comença aleshores a la seva pròpia freqüència de mostreig si
n'hi has donat una al mateix lloc, o si no a la freqüència de mostreig
global, i des d'allà segueix cada fitxer.

L'interruptor és desactivat quan el dispositiu no pot donar accés exclusiu.
- **Linux:** tria un dispositiu ALSA el nom del qual comenci per `hw:`. És la
  targeta de so mateixa. PulseAudio, PipeWire, JACK i els dispositius ALSA
  `default` o `plughw:` mesclen o converteixen, de manera que mai no són bit
  perfect.
- **Windows:** tria el dispositiu al sistema **WASAPI**. S'obre en mode
  exclusiu.
  - A la configuració de so de Windows, les propietats **Avançat** del
    dispositiu han de tenir activada l'opció *Permet que les aplicacions
    prenguin el control exclusiu d'aquest dispositiu* (és activada per
    defecte).
  - Mentre sona, cap altre programa no pot usar el dispositiu.
- **macOS:** tria el dispositiu a **Core Audio**. S'obre en mode hog.
  - La freqüència de mostreig del dispositiu s'ajusta a la de la pista, i el
    seu format al format enter més ampli que ofereix a aquella freqüència (els
    ajustos que mostra Configuració d'Àudio i MIDI).
  - Es retornen quan l'aplicació deixa d'usar el dispositiu.
  - Dos dispositius amb exactament el mateix nom no es poden fer bit perfect.

## Què passa en un dispositiu bit perfect {#what-happens-on-a-bit-perfect-device}

- **Accés exclusiu.** Cap altra cosa de l'ordinador no pot reproduir al
  dispositiu mentre l'aplicació l'usa. Si es refusa l'accés exclusiu, el
  dispositiu continua sonant, compartit, i el distintiu BP queda apagat.
- **La freqüència segueix el fitxer.** El dispositiu es torna a obrir a la
  freqüència de mostreig d'una pista quan la pista comença:
  - **Des del silenci:** reprodueixes una pista, en reprens una carregada en
    pausa, preescoltes o dispares un cartutx mentre no sona res al dispositiu.
    Les pistes que només esperen (la pista següent de cada reproductor) es
    tornen a preparar a la freqüència nova.
  - **Entre dues pistes d'un reproductor:** la pista en antena s'atura (al seu
    final, o a l'instant amb **Play / Següent** i **Anterior**), el dispositiu
    es torna a obrir i comença la pista següent. Aquest és el breu silenci de
    [Què costa](#what-it-costs).
  - El dispositiu només es torna a obrir quan ha reproduït tot el que tenia a
    la memòria intermèdia, de manera que el final de la pista anterior no es
    talla mai. Llavors tornar a obrir triga el que necessita el dispositiu per
    iniciar-se. Tot plegat són normalment unes quantes desenes de
    mil·lisegons; l'inici es retarda aquest temps. L'espera de la memòria
    intermèdia mai no és més llarga de mig segon (`tuning.play_out_max_ms` al
    fitxer de configuració, de 0 a 2000 ms), digui el que digui el dispositiu.
- **Quan la freqüència no pot seguir.** El dispositiu conserva la seva
  freqüència, i una pista a una altra freqüència es converteix durant tota la
  seva durada (no és bit perfect), quan:
  - alguna altra cosa sona al dispositiu quan la pista comença: un altre
    reproductor (o un en pausa amb una pista), un cartutx o una preescolta;
  - el dispositiu ja ha refusat aquella freqüència abans; conserva l'anterior;
  - la pista encara no s'ha analitzat, de manera que no es coneix la seva
    freqüència.

  El canvi entre les dues pistes continua sent sense fos encadenat.
- **Sense processament, quan res no el demana.** Les mostres passen sense
  canvis mentre no hi hagi cap fos en marxa (Stop amb fos, les rampes de pausa
  i represa, la breu aturada sense espetec d'una pista) i no soni res més a les
  mateixes sortides (un altre reproductor, un cartutx, un to de prova). El
  volum no és cap problema: una sortida bit perfect el manté al 100 %.

## DSD {#dsd}

Un fitxer DSD normalment es reprodueix convertit a PCM, com qualsevol altre
fitxer. Un dispositiu bit perfect pot, en canvi, rebre el flux DSD sense
canvis.

**Els tres modes.** A la vista Avançat de Configuració → Sortides d'àudio,
cada dispositiu que utilitza una sortida té una opció **DSD** sota el seu
interruptor bit perfect:
- **Convertir a PCM** (per defecte): el DSD es converteix, com en qualsevol
  altre dispositiu.
- **DoP** (DSD over PCM): els bits DSD viatgen dins de mostres PCM de 24 bits,
  que la majoria de convertidors compatibles amb DSD reconeixen. Funciona en
  tots els sistemes.
- **DSD natiu** (només Linux): DSD en brut, per a dispositius ALSA `hw:` el
  controlador dels quals declara un format de mostra DSD.

Només s'ofereixen els modes que el dispositiu pot acceptar, i una línia sota
l'opció diu per què no els altres: el dispositiu no està connectat, bit
perfect està desactivat, el dispositiu no es pot obrir en exclusiva, el DSD
natiu necessita Linux, o el dispositiu no accepta DSD natiu. Un mode desat
per a un dispositiu que ara no el pot acceptar apareix com a PCM, que és el
que sona; el mode desat torna quan el dispositiu el pot acceptar de nou.
Canviar un mode, l'ajust de mescla o el silenci DSD s'aplica quan no hi sona
res al dispositiu, com els altres ajustos de sortida.

**Quan el DSD surt sense canvis.** Totes aquestes condicions s'han de complir
quan comença la pista:
- el dispositiu és bit perfect, amb accés exclusiu, i el seu mode és DoP o
  DSD natiu;
- la pista és DSD (DSF o DFF), mono o estèreo, i s'ha analitzat (així se'n
  coneix la freqüència DSD);
- no hi sona res més al dispositiu (un altre reproductor, un cartutx, un to
  de prova);
- la pista no té guany de [normalització de la
  sonoritat](loudness.md#bit-perfect-outputs): una carregada (per exemple en
  pausa) amb un guany abans que el dispositiu passés a bit perfect el conserva,
  de manera que sona convertida, sense **DSD** ni **BP**;
- el dispositiu accepta el flux. DoP necessita una freqüència de dispositiu
  igual a la freqüència DSD dividida per 16 (176,4 kHz per a DSD64, 352,8 kHz
  per a DSD128, 705,6 kHz per a DSD256) i un format de 24 o 32 bits. El DSD
  natiu necessita un dispositiu que accepti el format DSD a aquella
  freqüència.

Quan el DSD natiu acaba, el dispositiu torna a PCM a la freqüència que tenia
abans de la pista DSD, perquè molts convertidors accepten DSD natiu a
freqüències a les quals no poden reproduir PCM (cap convertidor no reprodueix
PCM a la freqüència a la qual funciona el DSD512). Una pista que continua com
a PCM conserva la freqüència del flux DSD quan el dispositiu l'accepta com a
PCM, i si no continua a la freqüència anterior des d'on era, com tota la resta
que sona en aquell dispositiu; un fos en curs allà acaba a l'instant. El
dispositiu mai no es queda en una freqüència que refusa: si no s'obre cap
freqüència (per exemple, si el dispositiu s'ha desconnectat en aquell moment),
la sortida es perd fins que el reintent automàtic la torna a obrir a la
freqüència anterior.

En cas contrari, la pista es converteix a PCM i el registre en diu el motiu
(per exemple «something else plays on the device» o «the device refused
705600 Hz»). La preescolta i els cartutxos sempre es converteixen.

Mentre el DSD surt sense canvis:
- el distintiu de la capçalera diu **DSD** en lloc de **BP**;
- els mesuradors mostren el nivell de la conversió a PCM de la mateixa pista,
  de manera que funcionen com sempre;
- Stop i Stop amb fos aturen la pista a l'instant, sense fos, ja que un flux
  DSD no es pot esmorteir;
- la pausa i la represa també actuen a l'instant, sense rampa.

**Silenci als extrems.** Cada inici, final i pas a PCM envia primer silenci
DSD (200 ms per defecte), perquè el convertidor s'enganxi sense espetec.
L'excepció és una pista DSD que continua un flux del mateix tipus i
freqüència DSD el silenci del qual encara s'està emetent: el convertidor
encara està enganxat, de manera que comença sense silenci addicional. Una
pista comença, doncs, aquest temps més tard, i un pas a PCM deixa un buit
d'aquesta durada (un dispositiu DSD natiu també reprodueix el que té a la
memòria intermèdia abans de tornar-se a obrir com a PCM, uns quants
mil·lisegons més). És **Silenci DSD** a Configuració → Sortides d'àudio,
Avançat (de 0 a 2000 ms).

**Quan una altra font necessita el dispositiu.** **Quan una altra font
necessita una sortida DSD** a Configuració → Sortides d'àudio tria què passa
quan un altre reproductor, un cartutx o un to de prova comença al mateix
dispositiu:
- **Continuar la pista DSD en PCM** (per defecte). El flux passa a PCM
  després del silenci DSD, i la pista continua, convertida, des d'on era. El
  mateix passa amb la pista que segueix per si sola (vegeu més avall).
- **Mantenir el DSD i silenciar les altres fonts.** Res no interromp el flux
  DSD. Les altres fonts dirigides al dispositiu es silencien fins que acaba la
  pista DSD, i el reproductor mostra mentrestant un distintiu **Altres
  silenciades**. La pista següent del propi reproductor
  comença quan acaba la pista DSD: una pista PCM després del silenci DSD, una
  pista DSD del mateix tipus i freqüència DSD a l'instant, continuant el flux.

**Un àlbum no es queda en DSD amb l'ajust per defecte.** Amb *Continuar la
pista DSD en PCM*, només una pista DSD que comença en un dispositiu lliure
surt com a DSD. Les pistes que el reproductor inicia per si sol després (al
final d'una pista) comencen des d'una
precàrrega, que sempre és PCM, de manera que el dispositiu passa a PCM i
sonen convertides. Una pista que inicies tu mateix (Play, doble clic) torna a
sortir com a DSD quan el dispositiu és lliure o el flux DSD anterior encara
és en el seu silenci a la mateixa freqüència DSD. Per mantenir un àlbum DSD
sencer com a DSD, tria *Mantenir el DSD i silenciar les altres fonts*.
Aleshores cada pista del reproductor surt com a DSD, i la següent comença
quan acaba l'anterior.

Si el dispositiu es perd mentre sona DSD i torna sense poder portar-lo (per
exemple sense accés exclusiu), la pista continua com a PCM.

## El distintiu BP {#the-bp-badge}

El distintiu **BP** de la capçalera del reproductor només existeix en un
reproductor el dispositiu Main del qual funciona en bit perfect. S'encén (vora
i text verds; gris atenuat quan és apagat) mentre la pista actual arriba sense
canvis al seu dispositiu Main. Totes aquestes condicions s'han de complir:

- el dispositiu és bit perfect i obert amb accés exclusiu;
- el dispositiu funciona a la freqüència de mostreig de la pista;
- la pista és PCM enter sense pèrdua (WAV, AIFF, FLAC, ALAC, WavPack o
  Monkey's Audio), mono o estèreo, de 24 bits com a màxim, i el format del
  dispositiu admet la seva mida de mostra (un fitxer de 24 bits en un
  dispositiu de 16 bits no és bit perfect). El DSD es converteix, de manera
  que mai no encén BP; quan surt sense canvis (vegeu [DSD](#dsd)) el
  distintiu diu **DSD**;
- la pista s'ha analitzat, ja que així se'n coneixen la freqüència i la mida
  de mostra. Les pistes que una versió anterior va analitzar obtenen el format
  quan s'analitzen de nou (l'avís d'inici, o Configuració → Anàlisi), o tan bon
  punt un reproductor les mostra o un cartutx les conté;
- la pista sona a guany unitari: una sortida bit perfect no es normalitza mai,
  però una pista que va començar amb un guany de normalització abans que el seu
  dispositiu passés a bit perfect conserva aquell guany fins que s'atura, i BP
  es queda apagat per a ella;
- el volum es manté al 100 % en una sortida bit perfect, no hi ha cap fos en
  marxa (Stop amb fos, les rampes de pausa i represa i la breu aturada sense
  espetec d'una pista són els únics fosos que queden en aquest reproductor), i
  no hi sona res més a les mateixes sortides.

Alguns fitxers mai no es mostren com a bit perfect:
- **Fitxers amb pèrdua** (MP3, AAC, Ogg Vorbis, Opus): les mostres
  decodificades no són els valors enters que accepta un dispositiu.
- **Fitxers de més de 24 bits:** el mesclador treballa en coma flotant de 32
  bits, que porta 24 bits exactament.
- **Fitxers de més de dos canals:** es mesclen a estèreo.

## Comprovar-ho tu mateix {#checking-it-yourself}

Per verificar una cadena de punta a punta:

1. Connecta la sortida digital del dispositiu (S/PDIF, AES o loopback USB) a
   un enregistrador que capturi bit a bit.
2. Reprodueix un fitxer de prova sense pèrdua sense que soni res més.
3. Enregistra'l.
4. Compara l'enregistrament amb el fitxer. Per exemple, amb SoX, inverteix-ne
   un i mescla'ls: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`
   després d'alinear-ne els inicis. Totes les mostres de la diferència han de
   ser zero.
5. Comprova un canvi de freqüència: en mode Continu, amb **Mescla automàtica al
   punt MIX** activada, reprodueix una pista de 44,1 kHz que tingui un punt MIX
   seguida d'una de 96 kHz. La primera sona fins al seu cue-out sense fos;
   després d'un breu silenci el convertidor mostra 96 kHz a la segona. El
   distintiu BP s'encén a totes dues.
6. Després reprodueix dues pistes a la mateixa freqüència: la segona segueix la
   primera sense buit ni superposició.

Les proves automàtiques del projecte comproven la mateixa propietat dins de
l'aplicació, en un dispositiu simulat.

### DSD en un convertidor real {#dsd-on-a-real-converter}

Les proves automàtiques comproven el DSD només en dispositius simulats. El
projecte no ha provat DoP ni DSD natiu en un convertidor real. Per comprovar-ne
un:
1. Posa el dispositiu en **DoP** (o **DSD natiu** a Linux) i
   reprodueix un fitxer DSD sense que soni res més. La capçalera ha de
   mostrar **DSD**, i la pantalla del propi convertidor hauria de mostrar la
   freqüència DSD (per exemple DSD64) en lloc d'una freqüència PCM. Un
   convertidor que mostra una freqüència PCM o reprodueix soroll no reconeix el
   flux: torna a **Convertir a PCM**.
2. Escolta si hi ha un espetec o una ràfega de soroll a l'inici, en fer Stop,
   al final de la pista. Un espetec vol dir que el
   convertidor necessita un **Silenci DSD** més llarg (Configuració → Sortides
   d'àudio, Avançat).
3. Inicia un cartutx o un altre reproductor al mateix dispositiu, una vegada
   amb cada ajust de mescla, i comprova el comportament descrit més amunt.
4. A Linux, per comprovar el DSD natiu sense l'aplicació, executa
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test it -- --ignored conformance::native_dsd_on_a_real_device`.
   Obre el dispositiu en DSD natiu a DSD64 i reprodueix un segon de silenci
   DSD. Ha de passar, i el convertidor hauria d'enganxar-se a DSD64.
5. Amb fitxers DSD a `test-music/`, `cargo test --release -p fp-engine --test it dsd_real_music:: -- --ignored`
   els reprodueix a través del motor en un dispositiu simulat i compara les
   paraules amb els bytes del fitxer (vegeu [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
