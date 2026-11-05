# Android

Fauste Player també s'executa en mòbils i tauletes Android (Android 10 o
posterior), sobre el mateix nucli d'emissió que l'escriptori: els
reproductors, les llistes, la cartutxera, els marcadors, la normalització de
la sonoritat i totes les regles de reproducció funcionen de la mateixa manera.

## Instal·lació {#install}

Una versió que inclou l'aplicació té `fauste-player-<version>-android.apk` a
la seva pàgina de **Releases** (amb el seu `.sha256`). Obre'l al mòbil i
permet que el navegador o el gestor de fitxers instal·lin aplicacions quan
Android ho demani. Per actualitzar, instal·la l'APK més nou de la mateixa
manera: les teves llistes i ajustos es conserven. L'aplicació encara no és a
Google Play.

## Primer inici {#first-start}

- **Accés a la música.** Permet l'accés a la música (i a les notificacions, a
  Android 13 i posteriors). Sense això el navegador de música es queda buit.
- **L'emissió continua.** Mentre sona un reproductor o un cartutx, es manté una
  notificació i l'àudio continua amb la pantalla apagada o l'aplicació en
  segon pla. Com a l'escriptori, res no surt a l'antena per si sol després
  d'un inici o d'una fallada.
- **Enrere i tancament.** Enrere primer tanca el que és obert (el navegador de
  música, Configuració, Quant a, un reproductor obert, una pàgina de la
  pestanya Biblioteca). Quan no queda res per tancar, Enrere deixa l'aplicació
  de banda mentre alguna cosa sona (un reproductor, un cartutx o una
  preescolta CUE) i continua reproduint; quan no sona res, la desa i la tanca.
  Fer lliscar l'aplicació al selector d'aplicacions fa el mateix: un lliscament
  no atura mai el que sona, i l'aplicació es tanca quan queda en silenci.
- **El so d'altres aplicacions.** Quan una altra aplicació pren l'àudio, Fauste
  Player continua reproduint i ho diu a la barra d'estat de baix.

## La pantalla {#the-screen}

En un **mòbil** la barra superior té quatre pestanyes, **Directe**,
**Cartutxera**, **Llistes** i **Biblioteca**, mostrades com a icones (la triada
també amb el seu nom quan hi ha lloc), després el **cadenat** i **⋮** (Més:
Configuració i Quant a). El rellotge es mostra quan hi ha lloc. Girat de
costat, la pantalla de Llistes posa els seus xips de reproductor (P1, P2…) a
la barra superior, i la barra d'estat de baix només es mostra quan té alguna
cosa per dir.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Directe en un mòbil: dos reproductors com a targetes, una sencera i l'inici de la següent, i la cartutxera a sota">
<img src="../../images/android/2-carts.png" width="180" alt="Cartutxera en un mòbil: la pàgina General girada en vertical, dos cartutxos per fila">
<img src="../../images/android/3-playlists.png" width="180" alt="Llistes en un mòbil: els xips de P1 a P4, les pestanyes de llista, la llista de pistes i la targeta del reproductor a baix">
<img src="../../images/android/4-music.png" width="180" alt="Música del dispositiu en un mòbil: la carpeta Demo amb tres cançons marcades i la barra per afegir-les a la llista Morning de P1">
</p>

- *Directe:* cada reproductor com a targeta, i la cartutxera. Cada targeta té,
  de dalt a baix, el nom del reproductor, l'estat, Single/Cont i CUE; la
  caràtula, el títol i l'artista i la pista següent; la forma d'ona (un toc
  salta); després el temps transcorregut i la durada de la pista amb el temps
  restant en dígits grans sobre el transport (Play / Següent, Anterior, Pausa,
  Stop, Reiniciar, Stop amb fos, Stop al final de la pista actual) amb botons
  de la mida d'un dit, i un mesurador prim amb el fader de volum (el nivell
  del fader es mostra a sota). El temps restant acaba on acaben els botons,
  més petit per a una pista llarga. A la majoria de mòbils el mesurador mostra
  la seva escala a l'esquerra, i en un de més ample (uns 406 dp o més) el
  fader també mostra els seus nivells; en un mòbil estret (per sota d'uns
  384 dp) cap dels dos ho fa, i els botons es fan una mica més petits perquè
  hi càpiguen. Els botons funcionen mentre un altre dit sosté el fader, i un
  toc compta encara que el dit es mogui una mica o es quedi premut un moment.
  Sostingut en vertical, les targetes són a dalt, una de sencera i l'inici de
  la següent (desplaça't per veure'n més), i la cartutxera a sota. Girat de
  costat, cada reproductor és una fila a l'esquerra (el seu nom i estat, el
  títol, el temps restant, Play / Següent, Pausa i Stop; toca la fila en un
  altre lloc per obrir el reproductor a Llistes) i la cartutxera és a la
  dreta. Toca **CARTUTXERA** a la seva capçalera per plegar-la: només queda la
  capçalera, a baix (les pestanyes de pàgina i *Aturar-ho tot* encara
  funcionen), i els reproductors prenen l'espai; toca-la de nou per
  desplegar-la. La pestanya Cartutxera sempre mostra tota la cartutxera. Una
  pàgina de cartutxos massa ampla per a l'espai es mostra en menys columnes,
  en el mateix ordre.
- *Cartutxera:* la pàgina de cartutxos mostrada, omplint la pantalla. Llisca
  lateralment per mostrar la pàgina següent o l'anterior. Sostinguda en
  vertical, una pàgina més ampla que alta es gira (8 × 2 es mostra com 2 × 8),
  en el mateix ordre.
- *Llistes:* els xips de dalt (P1, P2…) trien el reproductor; sota seu hi ha
  les seves pestanyes de llista (+ en fa una de nova), la llista de pistes i, a
  baix (a la dreta quan el mòbil és de costat), la targeta del reproductor amb
  una forma d'ona més gran per preparar una pista (es fa més curta quan manca
  espai, de manera que els botons sempre es veuen). Una pulsació llarga en una
  fila n'obre el menú, que també té **Pujar**, **Baixar** i **Moure a un altre
  reproductor** (la pista va just després de la pista següent d'aquell
  reproductor, a la llista que mostra; la pista en antena es queda). Arrossega
  una fila pel seu ≡ per moure-la. El peu té **Afegir** i, darrere de **⋯**,
  **Reiniciar**. Una pulsació llarga a la forma d'ona estableix un marcador en
  aquell punt (cue-in, final de la intro, inici de l'outro, MIX, cue-out) o en
  treu un que has posat; els marcadors només s'editen aquí, no a Directe.
- *Biblioteca:* la biblioteca de pistes i la música del dispositiu (vegeu
  [Biblioteca](#library)).

Enrere des d'una altra pantalla va a Directe; a Biblioteca primer torna a la
llista de categories (a Música del dispositiu, primer una carpeta amunt).

Quan res no es mou a la pantalla, l'aplicació la dibuixa un cop per segon (pel
rellotge) per estalviar bateria; un mesurador que encara cau després d'una
pausa o d'un stop la manté dibuixant fins que el mesurador és a baix.

**El cadenat** protegeix dels tocs accidentals durant un programa. Bloquejat
(el cadenat tancat i il·luminat), els faders de volum només mostren el nivell
i les formes d'ona ni salten ni editen marcadors; es veuen atenuats. Un
pessic i un arrossegament encara amplien i mouen una forma d'ona, i Play, el
transport, els cartutxos i les llistes sempre funcionen. El bloqueig es manté
després d'un reinici i es desa amb la sessió, de manera que carregar una altra
sessió pren el bloqueig d'aquella sessió; l'API remota i el MIDI no es
bloquegen.

Un arrossegament que comença en un fader de volum el mou; un que comença en
qualsevol altre lloc desplaça, de manera que desplaçar mai no canvia un nivell
a l'antena. Un toc en una forma d'ona hi salta, un pessic de dos dits hi amplia
al voltant dels dits, un arrossegament d'un dit mou una vista ampliada i **Vista
completa** mostra de nou tota la pista. En una llista un dit desplaça; per
moure una pista, fes servir el seu ≡ o el menú de la fila.

En una **tauleta** la disposició és la de l'escriptori (vegeu
[Disposició](layout.md)), amb elements més fàcils de tocar.

Una pulsació llarga obre el que obre un clic dret a l'escriptori: el menú de
fila, els menús de capçalera, el menú d'un cartutx (**Editar…** tria el fitxer
d'un cartutx buit). En un mòbil, Configuració omple la pantalla i llista les
seves seccions; toca'n una per obrir-la, amb el nom de cada ajust sobre el seu
control. La fletxa o Enrere torna a la llista, i Enrere de nou (o **Tancar**)
tanca Configuració. Editar un cartutx n'obre la secció directament.

Quant a → **Llicències de tercers** obre els avisos de les biblioteques amb
què es construeix l'aplicació en un navegador o visor d'HTML.

## Biblioteca {#library}

En un **mòbil** la pestanya **Biblioteca** s'obre amb una llista de
categories: **Totes les pistes**, les teves categories (cada subcategoria sota
la seva mare, amb el seu color i quantes pistes conté), **Sense categoria** i
**Música del dispositiu**. La biblioteca és la mateixa que mostra la [finestra
de la biblioteca](library.md) de l'escriptori: cada fitxer afegit a una llista,
a un cartutx o a la biblioteca hi és. Les categories es creen, se'ls canvia el
nom i el color i s'eliminen a **Configuració → Biblioteca**.

**Les pistes d'una categoria.** Toca una categoria per obrir-ne les pistes
(amb les de les seves subcategories). La fletxa de la seva capçalera, o
Enrere, torna a les categories. Una altra pestanya i tornar troba la llista
com l'has deixada. Obrir una categoria des de les categories comença amb una
cerca buida. La cerca filtra la llista per títol, artista, àlbum i les altres
etiquetes que cerca la finestra de la biblioteca; **Tota la biblioteca** cerca
a totes les categories. Cada fila mostra el títol (o el nom del fitxer),
l'artista, la durada i les marques de la finestra per a un fitxer absent,
il·legible o encara no analitzat.

Toca files per marcar-les. Mentre alguna cosa és marcada, la barra de Música
del dispositiu apareix a baix: quantes, el reproductor i la llista on van
(toca-la per triar-ne una altra), **Afegir al final** i **Afegir després de la
següent**, com a Música del dispositiu. Les pistes van en l'ordre en què les
has marcat, i afegir esborra les marques. Tornar a les categories també les
esborra.

**El menú de la fila.** Una pulsació llarga en una fila n'obre el menú:
**Afegeix a ▸** (la llista d'un reproductor, al final), **Preescolta**,
**Mou a la categoria ▸**, **Tornar a analitzar** i **Treu de la biblioteca**,
que pregunta com a l'escriptori (vegeu [Treure
pistes](library.md#removing-tracks)). Mentre hi ha files marcades, una
pulsació llarga en una altra fila també la marca i el menú actua sobre cada
fila marcada, en l'ordre en què les has marcat; llavors **Preescolta** no
s'ofereix. Les marques es mantenen després de l'acció del menú.

**Preescolta.** El 🎧 a la dreta d'una fila reprodueix la seva pista des del
seu cue-in per la sortida de preescolta de la biblioteca; el 🎧 d'una altra
fila canvia a aquella pista. Una franja prima sobre la llista mostra el
títol, el temps restant, **Pausa**/**Reproduir** i **Stop** mentre sona (o és
en pausa), i una indicació altrament: sempre manté el seu lloc, de manera que
les files no es mouen quan una preescolta comença o s'atura. El 🎧 és atenuat
quan el fitxer és absent o no es pot reproduir. La preescolta s'atura quan la
seva pista surt de la biblioteca i quan deixes la pestanya Biblioteca (per una
altra pestanya, o amb Enrere); a les categories o a Música del dispositiu
continua, però la seva franja només es mostra a la llista de pistes.

A diferència de l'escriptori, Android (un mòbil o una tauleta) reprodueix la
preescolta de la biblioteca per la sortida predeterminada quan no s'hi ha triat
cap sortida: a **Configuració → Sortides d'àudio → Biblioteca** l'elecció
buida diu *Predeterminat del sistema*, i el seu **Provar Cue** hi sona. Tria-hi
una altra sortida (uns auriculars amb cable, per exemple) per preescoltar lluny
de l'antena. La sortida predeterminada normalment també és la dels
reproductors i la de la cartutxera, de manera que la preescolta es pot sentir a
l'antena: mentre alguna cosa sona en aquella sortida, el 🎧 pregunta primer
(**Preescoltar igualment** o **Cancel·lar**), i una preescolta que no has
confirmat s'atura quan alguna cosa hi comença, com fa el CUE d'un reproductor
(vegeu [CUE (preescolta)](players.md#cue-pre-listen)).

**Afegir música del dispositiu a una categoria.** El **＋** de la capçalera
d'una llista de pistes obre Música del dispositiu per a aquella categoria. La
seva barra només té llavors **Afegeix a &lt;categoria&gt;** (**Afegeix a Sense
categoria** des de *Totes les pistes* o *Sense categoria*), que afegeix els
fitxers marcats a la biblioteca en aquella categoria i torna a la llista; un
fitxer que ja és a la biblioteca s'hi uneix si no té categoria i conserva la
que té altrament, com a l'escriptori. Música del dispositiu conserva les marques
entre visites, de manera que els fitxers marcats en una visita anterior també
s'afegeixen: comprova primer el recompte de la barra. **Tornar a la llista**, a
dalt, o Enrere torna a la llista sense afegir. Música del dispositiu oberta des
de les categories té **Tornar a les categories** a dalt.

**La carpeta del dispositiu d'una categoria.** Una categoria pot prendre els
seus fitxers d'una carpeta de la música del dispositiu. Al seu menú (una
pulsació llarga en ella a **Configuració → Biblioteca**, o a la finestra de la
biblioteca en una tauleta), **Usa una carpeta del dispositiu…** obre Música del
dispositiu per triar la carpeta (**Fes servir aquesta carpeta**); el menú de la
categoria llavors la nomena, sobre els seus elements de carpeta. Res no vigila
la carpeta: quan hi has copiat música nova, toca el **↻** de la capçalera de la
llista de pistes de la categoria (o **Rellegeix la carpeta del dispositiu** al
seu menú). Cada fitxer que Música del dispositiu llista a la carpeta i a les
seves subcarpetes s'afegeix a la categoria, amb les regles anteriors (un fitxer
que ja és en una altra categoria la conserva), i un avís diu quants fitxers
s'han llegit. Els fitxers que han sortit de la carpeta es queden a la
biblioteca; **Tornar a analitzar** marca com a absents els que han perdut el
fitxer. **Canvia la carpeta del dispositiu…** i **Oblida la carpeta del
dispositiu** són al mateix menú; una carpeta pertany a una categoria.

### Música del dispositiu {#device-music}

**Música del dispositiu** mostra els fitxers d'àudio del dispositiu per
**Carpetes** (amb el camí sobre la llista; toca una part per tornar amunt),
**Artistes** o **Àlbums**, i una cerca per títol, artista, àlbum i nom de
fitxer. Cada fila mostra el títol, l'artista i la durada; toca files per
marcar-les. Mentre alguna cosa és marcada, una barra a baix mostra quantes, el
reproductor i la llista on van (toca-la per triar-ne una altra), **Afegir al
final** i, quan la pista següent d'aquell reproductor és en aquella llista,
**Afegir després de la següent**. Les pistes van en l'ordre en què les has
marcat. Un fitxer del dispositiu no té preescolta.

**Afegir** al peu d'una llista, el fitxer d'un cartutx i Configuració →
Llistes → Carpeta de música obren Música del dispositiu en un mòbil (en una
tauleta, la finestra del **navegador de música**, per carpetes). Marca fitxers
i toca **Afegir**, toca un fitxer per a un cartutx, o **Fes servir aquesta
carpeta**; **Cancel·la** o Enrere torna a on eres. Copia música al dispositiu
(per exemple a la seva carpeta *Music*) per veure-la allà. Fauste Player només
llegeix la teva música: les etiquetes i les caràtules no s'escriuen, i les
llistes i pàgines de cartutxos encara no s'importen ni s'exporten a Android.

En una **tauleta** la biblioteca és la [finestra de la
biblioteca](library.md) de l'escriptori, oberta des de la barra superior.

## Sortides {#outputs}

Configuració → Sortides d'àudio llista les sortides del mòbil per tipus: l'altaveu,
uns auriculars amb cable, una interfície USB, un altaveu Bluetooth («JBL Flip
(Bluetooth)»). Cada reproductor, la cartutxera i els seus CUE en trien una,
com a l'escriptori, i també ho fa la preescolta de la biblioteca (vegeu
[Biblioteca](#library)). Un altaveu Bluetooth que s'apaga i es torna a encendre
es troba de nou per la seva ruta; mentre és fora, els seus reproductors es
comporten com amb un dispositiu absent a l'escriptori. Si dues sortides poden
sonar alhora depèn del mòbil.

## No és a Android {#not-on-android}

Les superfícies de control MIDI, el control remot, l'edició d'etiquetes i
caràtules, i els fitxers de llistes i de pàgines de cartutxos són, de moment,
funcions de l'escriptori. Un teclat connectat a una tauleta funciona com a
l'escriptori (vegeu [Teclat](keyboard.md)).
