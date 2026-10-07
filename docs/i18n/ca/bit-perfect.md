# Sortida bit perfect

Un dispositiu **bit perfect** rep les mostres de cada fitxer exactament tal
com són al fitxer: la mateixa freqüència de mostreig, els mateixos valors,
sense remostreig, canvi de volum ni mescla. És útil per a cadenes de
monitoratge i enllaços digitals, on cal evitar qualsevol processament a
l'ordinador.

## Posar un dispositiu en bit perfect {#setting-a-device-bit-perfect}

1. A **Configuració → Sortides d'àudio**, tria el dispositiu explícitament
   per a la sortida Main d'un reproductor (o de la cartutxera). Un reproductor
   que es deixa al predeterminat del sistema no pot ser bit perfect. Un
   dispositiu que cap sortida no utilitza ja perd el seu interruptor bit
   perfect i el seu mode DSD la propera vegada que s'inicia l'aplicació.
2. Tria **Avançat** a dalt de la secció. A **Ajustos per dispositiu**,
   activa **Bit perfect** al costat del dispositiu.
3. Reinicia l'aplicació.

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
- **La freqüència segueix el fitxer.** Quan no hi sona res al dispositiu i
  comença una pista a una altra freqüència de mostreig, el dispositiu es torna
  a obrir a aquella freqüència.
  - Això passa quan reprodueixes una pista, en reprens una carregada en pausa,
    preescoltes o dispares un cartutx. Les pistes que només esperen (la pista
    següent de cada reproductor) es tornen a preparar a la freqüència nova.
  - Tornar a obrir triga el que necessita el dispositiu per iniciar-se
    (normalment unes quantes desenes de mil·lisegons). L'inici es retarda
    aquest temps.
  - Mentre hi sona alguna cosa al dispositiu, la freqüència mai no canvia. Una
    pista a una altra freqüència que comença aleshores (per exemple una pista
    de 48 kHz mesclada després d'una de 44,1 kHz, o una pista iniciada mentre
    un altre reproductor o un cartutx sona al mateix dispositiu) es converteix
    durant tota la seva durada, i no és bit perfect.
  - Si el dispositiu refusa una freqüència, conserva l'anterior i la pista es
    converteix.
- **Sense processament, quan res no el demana.** Les mostres passen sense
  canvis mentre es compleixin totes aquestes condicions:
  - el volum del reproductor és al 100 %;
  - no hi ha cap fos en marxa;
  - no hi sona res més a les mateixes sortides (un altre reproductor, un
    cartutx, un to de prova).

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
Canviar un mode, l'ajust de mescla o el silenci DSD necessita un reinici,
com els altres ajustos de sortida.

**Quan el DSD surt sense canvis.** Totes aquestes condicions s'han de complir
quan comença la pista:
- el dispositiu és bit perfect, amb accés exclusiu, i el seu mode és DoP o
  DSD natiu;
- la pista és DSD (DSF o DFF), mono o estèreo, i s'ha analitzat (així se'n
  coneix la freqüència DSD);
- el volum del reproductor és al 100 %;
- no hi sona res més al dispositiu (un altre reproductor, un cartutx, un to
  de prova);
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
- el volum s'ha de mantenir al 100 %: la informació emergent del fader ho
  diu. Moure'l passa la pista a PCM (vegeu més avall);
- Stop i Stop amb fos aturen la pista a l'instant, sense fos, ja que un flux
  DSD no es pot esmorteir. Prémer Play en una altra pista mentre sona la talla
  de la mateixa manera en lloc de fer un fos encadenat;
- la pausa i la represa també actuen a l'instant, sense rampa.

**Silenci als extrems.** Cada inici, final i pas a PCM envia primer silenci
DSD (200 ms per defecte), perquè el convertidor s'enganxi sense espetec.
L'excepció és una pista DSD que continua un flux del mateix tipus i
freqüència DSD el silenci del qual encara s'està emetent: el convertidor
encara està enganxat, de manera que comença sense silenci addicional. Una
pista comença, doncs, aquest temps més tard, i un pas a PCM deixa un buit
d'aquesta durada. És **Silenci DSD** a Configuració → Sortides d'àudio,
Avançat (de 0 a 2000 ms).

**Quan una altra font necessita el dispositiu.** **Quan una altra font
necessita una sortida DSD** a Configuració → Sortides d'àudio tria què passa
quan un altre reproductor, un cartutx o un to de prova comença al mateix
dispositiu (moure el fader del propi reproductor és l'excepció: sempre passa
la pista a PCM):
- **Continuar la pista DSD en PCM** (per defecte). El flux passa a PCM
  després del silenci DSD, i la pista continua, convertida, des d'on era. El
  mateix passa amb la pista que segueix per si sola (vegeu més avall).
- **Mantenir el DSD i silenciar les altres fonts.** Res no interromp el flux
  DSD. Les altres fonts dirigides al dispositiu es silencien fins que acaba la
  pista DSD, i el reproductor mostra mentrestant un distintiu **Altres
  silenciades**. La pista següent del propi reproductor no se superposa:
  comença quan acaba la pista DSD, sense fos encadenat ni segue. Una pista PCM
  espera el silenci DSD; una pista DSD del mateix tipus i freqüència DSD
  continua el flux sense aquest silenci. Moure el fader continua passant la
  pista a PCM.

**Un àlbum no es queda en DSD amb l'ajust per defecte.** Amb *Continuar la
pista DSD en PCM*, només una pista DSD que comença en un dispositiu lliure
surt com a DSD. Les pistes que el reproductor inicia per si sol després (al
final d'una pista, en un segue o en un fos encadenat) comencen des d'una
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

El distintiu **BP** de la capçalera del reproductor s'encén mentre la pista
actual arriba sense canvis al seu dispositiu Main. Totes aquestes condicions
s'han de complir:

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
- el volum és al 100 %, no hi ha cap fos en marxa i no hi sona res més a les
  mateixes sortides.

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
2. Reprodueix un fitxer de prova sense pèrdua al 100 % sense que soni res
   més.
3. Enregistra'l.
4. Compara l'enregistrament amb el fitxer. Per exemple, amb SoX, inverteix-ne
   un i mescla'ls: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`
   després d'alinear-ne els inicis. Totes les mostres de la diferència han de
   ser zero.

Les proves automàtiques del projecte comproven la mateixa propietat dins de
l'aplicació, en un dispositiu simulat.

### DSD en un convertidor real {#dsd-on-a-real-converter}

Les proves automàtiques comproven el DSD només en dispositius simulats. El
projecte no ha provat DoP ni DSD natiu en un convertidor real. Per comprovar-ne
un:
1. Posa el dispositiu en **DoP** (o **DSD natiu** a Linux), reinicia i
   reprodueix un fitxer DSD al 100 % sense que soni res més. La capçalera ha de
   mostrar **DSD**, i la pantalla del propi convertidor hauria de mostrar la
   freqüència DSD (per exemple DSD64) en lloc d'una freqüència PCM. Un
   convertidor que mostra una freqüència PCM o reprodueix soroll no reconeix el
   flux: torna a **Convertir a PCM**.
2. Escolta si hi ha un espetec o una ràfega de soroll a l'inici, en fer Stop,
   al final de la pista i en moure el fader. Un espetec vol dir que el
   convertidor necessita un **Silenci DSD** més llarg (Configuració → Sortides
   d'àudio, Avançat).
3. Inicia un cartutx o un altre reproductor al mateix dispositiu, una vegada
   amb cada ajust de mescla, i comprova el comportament descrit més amunt.
4. A Linux, per comprovar el DSD natiu sense l'aplicació, executa
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   Obre el dispositiu en DSD natiu a DSD64 i reprodueix un segon de silenci
   DSD. Ha de passar, i el convertidor hauria d'enganxar-se a DSD64.
5. Amb fitxers DSD a `test-music/`, `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   els reprodueix a través del motor en un dispositiu simulat i compara les
   paraules amb els bytes del fitxer (vegeu [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
