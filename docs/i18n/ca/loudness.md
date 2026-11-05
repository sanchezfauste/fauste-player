# Normalització de la sonoritat

Les pistes vénen de moltes fonts i estan masteritzades a nivells molt
diferents: un senzill pop recent pot sonar el doble de fort que una gravació
dels anys setanta o una promoció parlada. La **normalització de la sonoritat**
reprodueix totes les pistes i tots els cartutxos amb la mateixa sonoritat
percebuda, de manera que l'operador no ha de moure el fader d'una pista a la
següent.

Està **activada per a les instal·lacions noves**; una configuració existent
conserva el seu ajust actual (cada configuració desada ja el guarda). Canvia-la
a [Configuració → Sonoritat](settings.md#loudness).

![Configuració, Sonoritat: normalització activada, amb l'objectiu, el sostre de true peak, el guany màxim i l'ajust màxim als seus valors per defecte](../../images/guide/settings-loudness.png)

## Com funciona {#how-it-works}

**Un guany fix per pista.** Cada pista rep un únic guany, en dB, decidit quan
la pista es carrega i mantingut fins que s'atura. No es comprimeix ni es
limita res: tota la pista es fa més forta o més fluixa en la mateixa
quantitat, de manera que la seva dinàmica es manté tal com es va masteritzar.
El fader del reproductor actua per sobre, com abans.

**La mesura.** L'anàlisi en segon pla mesura cada fitxer una vegada:

- la seva **sonoritat integrada**, en LUFS: com de fort sona tot el fitxer,
  tal com la defineixen ITU-R BS.1770 i EBU R128 (els passatges fluixos i el
  silenci es descarten, de manera que un fos final llarg o una pausa no la
  baixen);
- el seu **true peak**, en dBTP: el nivell més alt que assoleix el senyal
  entre mostres un cop convertit a analògic.

Els fitxers nous es mesuren amb la seva primera anàlisi. Els fitxers afegits
abans que es mesurés la sonoritat es mesuren en una **passada en segon pla**
que s'executa mentre l'anàlisi normal no té res a fer, sense tocar-ne els
marcadors. S'executa tant si la normalització és activada com desactivada,
de manera que les mesures són a punt quan l'actives. **Tornar a analitzar**
(al menú de la llista) torna a mesurar un fitxer, llegint-lo encara que la
seva mida i data no hagin canviat. Un fitxer de menys de 0,4 s no es mesura
mai, i un fitxer silenciós no té sonoritat: tots dos sonen sense canvis.

**El guany.** El guany porta la sonoritat mesurada a l'**Objectiu**, més
l'**Ajust** propi de la pista (vegeu [Per pista](#per-track-exclude-and-trim)):

> guany = objectiu − sonoritat mesurada + ajust

Una pista més forta que l'objectiu es baixa tant com calgui. Una pista més
fluixa es puja, però mai més del **Guany màxim**, i mai tant que el seu true
peak superés el **Sostre de true peak**: no hi ha limitador, de manera que el
sostre és el que evita que una pista pujada saturi. Una pista el true peak de
la qual ja és per sobre del sostre no es puja gens.

Exemples, amb els valors per defecte (objectiu −16 LUFS, sostre −1 dBTP, guany
màxim 12 dB):

| Pista | Mesurada | True peak | Guany |
|---|---|---|---|
| Un màster modern fort | −9,0 LUFS | +0,5 dBTP | −7,0 dB |
| Un màster antic més fluix | −20,0 LUFS | −10,0 dBTP | +4,0 dB |
| Una pista fluixa amb pics alts | −22,0 LUFS | −3,0 dBTP | +2,0 dB (limitat pel sostre) |
| Una gravació molt fluixa | −35,0 LUFS | −20,0 dBTP | +12,0 dB (limitat pel guany màxim) |

**Quan es decideix el guany.** Quan es carrega una pista: preparada com a
següent d'un reproductor, iniciada, carregada en pausa, preescoltada al CUE,
o disparada com a cartutx. Una pista que sona o és en pausa **conserva el seu
guany** fins que s'atura, durant salts i pauses. Canviar l'ajust, la mesura
d'una pista o el seu ajust s'aplica la propera vegada que es carrega la pista;
la pista següent que ja espera en un reproductor es torna a preparar amb el
guany nou, de manera que comença amb ell.

**Preescolta.** El CUE i la preescolta de cartutxos reprodueixen una pista amb
el mateix guany que rep a l'antena, de manera que el que sents als auriculars
coincideix amb el programa.

**Mesuradors.** Els mesuradors i la seva lectura de sonoritat mesuren després
del guany, de manera que mostren el nivell normalitzat. L'**Objectiu de
sonoritat** propi dels mesuradors (Configuració → Vúmetres) només acoloreix la
lectura; no canvia cap guany.

## Sortides bit perfect {#bit-perfect-outputs}

Un reproductor o cartutx la sortida del qual és [bit perfect](bit-perfect.md)
**no es normalitza mai**: sempre sona a guany unitari (0 dB), sigui quin sigui
l'ajust, perquè qualsevol guany canviaria les mostres que rep el dispositiu.
Amb la normalització activada, la seva capçalera mostra una insígnia ambre
**0.0 dB** el consell de la qual diu «Sortida bit perfect: les pistes sonen
sense alterar.» El mateix val per a la preescolta en un dispositiu Cue bit
perfect, mentre la sortida Main del reproductor, si no és bit perfect, encara
rep el guany.

Una pista que ja sona (o és en pausa) quan el seu dispositiu passa a bit
perfect conserva el guany amb què va començar, i la insígnia **BP** continua
apagada per a ella. La pista següent sona a 0 dB i bit perfect. Una pista DSD
en aquesta situació sona convertida a PCM amb el seu guany, mai com un flux
DSD sense canvis.

## El guany a la capçalera del reproductor {#the-gain-in-the-player-header}

Mentre un reproductor sona o és en pausa, una insígnia a l'esquerra de **BP**
mostra el guany de la pista en antena. La insígnia té una vora del seu color i
sempre la mateixa amplada. Passa-hi el cursor per veure'n el motiu.

| Insígnia | Significat |
|---|---|
| **+4.0 dB**, **−7.0 dB** (blau) | La pista es normalitza amb aquest guany. El consell és el desglossament del guany descrit més avall. Quan l'ajust, la mesura o l'ajust de la pista han canviat després de carregar-la, només diu que la pista conserva el guany fixat quan es va carregar, i mostra aquest guany. |
| **0.0 dB** (ambre) | La pista sona sense canvis. El consell és el mateix desglossament, i la seva línia **Guany automàtic** diu per què: encara no té mesura (o és massa curta per mesurar-se), la sortida és bit perfect, o la pista està exclosa de la normalització. |
| Cap insígnia | La normalització era desactivada quan la pista va començar, la pista és silenciosa, o el reproductor és aturat. Un reproductor bit perfect tampoc no en mostra mentre la normalització és desactivada. |

La insígnia mostra el guany que el reproductor va demanar quan la pista va
començar.

### El desglossament del guany {#the-gain-breakdown}

Allà on el programa explica el guany d'una pista mostra el mateix bloc de
línies, amb l'etiqueta a l'esquerra i el valor a la dreta: el consell de la
insígnia de guany, la finestra de sonoritat (sota els seus controls), l'editor
de cartutxos i el consell de pista de la taula de la llista.

| Línia | Significat |
|---|---|
| **Sonoritat integrada** | La sonoritat mesurada de la pista, en LUFS (o «sense mesurar»). |
| **True peak** | El seu true peak més alt, en dBTP (només quan s'ha mesurat). |
| **Objectiu** | La sonoritat a la qual es porta cada pista (Configuració → Sonoritat). |
| **Guany automàtic** | Objectiu menys sonoritat mesurada, abans de l'ajust i de qualsevol límit, o «cap» i per què: exclosa, sortida bit perfect, normalització desactivada, encara sense mesurar, o silenciosa. |
| **Ajust** | L'ajust manual de la pista en ús, quan no és 0 (amb el valor desat i el màxim quan un **Ajust màxim** reduït el limita). |
| **Guany desitjat** i **Limitat per** | Només quan el sostre de true peak o el guany màxim han frenat el guany: el guany automàtic més l'ajust (mostrat quan hi ha ajust) i el límit que l'ha aturat. |
| **Guany resultant** | Sempre l'últim, sota una línia i en negreta: el guany amb què sona la pista. 0.0 dB quan no es normalitza. |

Amb la normalització desactivada, una nota sobre el guany resultant diu que no
canvia res a l'antena i que l'ajust s'aplica quan s'activi.

## Per pista: excloure i ajustar {#per-track-exclude-and-trim}

Algunes pistes no han de seguir l'objectiu: una identificació de l'emissora
masteritzada forta a propòsit, un to de prova, una pista la mesura de la qual
no coincideix amb com sona (la veu sovint sona més forta que la música amb els
mateixos LUFS). Cada pista té dos ajustos per a això:

- **Excloure de la normalització:** la pista sempre sona sense canvis.
- **Ajust:** dB afegits al guany de la pista, de − a + l'**Ajust màxim**
  (Configuració → Sonoritat), en passos de 0,5 dB. Fes doble clic al control
  lliscant per tornar-lo a 0. L'ajust és atenuat mentre la pista és exclosa.

El control lliscant és un **ajust**: un desplaçament per sobre del guany
automàtic, de manera que marca 0 dB per a cada pista que no has editat. La
finestra i l'editor de cartutxos mostren per tant el [desglossament del
guany](#the-gain-breakdown) amb els seus controls: la línia **Guany
resultant** segueix el control lliscant mentre el mous. A la finestra els
controls van primer i el desglossament sota ells, de manera que el control
lliscant es queda on és mentre el desglossament creix o s'encongeix. Mentre la
pista sona, la finestra afegeix que un canvi s'aplica des de la seva propera
càrrega.

On canviar-los:

- **En una forma d'ona:** fes clic dret a la forma d'ona d'un reproductor (o
  a la de la seva finestra CUE) i tria **Sonoritat…**. La finestra anomena la
  pista i en mostra la mesura. Continua editant aquella pista encara que el
  reproductor passi a una altra; tanca-la amb el seu botó **×**. Es tanca per
  si sola si la pista es treu.
- **En una fila d'una llista:** fes clic dret a la fila i tria
  **Sonoritat…**: la mateixa finestra, per a la pista d'aquella fila.
- **A l'editor de cartutxos:** Configuració → Cartutxera, fes clic en un
  cartutx: els mateixos dos controls són sota **Aturar els altres cartutxos en
  disparar-lo**, amb la mesura sota l'etiqueta.
- **Remotament:** `PUT /api/v1/tracks/<id>/loudness` (vegeu [Control
  remot](remote-control.md)).

Els ajustos pertanyen a la pista i es desen amb ella. Un canvi s'aplica la
propera vegada que es carrega la pista, com qualsevol altre canvi.

**El consell de pista** de la taula de la llista mostra les etiquetes de la
pista i, sota una línia, el mateix [desglossament del guany](#the-gain-breakdown),
calculat per al reproductor al qual pertany aquella llista (un reproductor bit
perfect la reprodueix sense alterar).

## Triar els ajustos {#choosing-the-settings}

- **Objectiu.** −16 LUFS va bé per a la majoria de programes musicals a
  l'antena i en línia. −23 LUFS (EBU R128) o −24 LUFS (ATSC A/85) deixen més
  marge, però llavors les pistes fluixes necessiten pujades més grans, que el
  sostre i el guany màxim poden frenar. Tria un objectiu proper a com de fort
  ja és el teu material.
- **Sostre de true peak.** −1 dBTP deixa lloc per a la conversió i per als
  codificadors amb pèrdua aigües avall. Baixa'l quan un codificador de flux
  saturi.
- **Guany màxim.** Evita que una gravació molt fluixa (o gairebé silenciosa)
  es pugi fins al soroll. 0 dB només baixa les pistes fortes.
- **Ajust màxim.** Fins on pot arribar l'ajust per pista. Reduir-lo limita els
  ajustos ja fets sense canviar-los, fins que edites la sonoritat d'aquella
  pista: l'edició llavors desa l'ajust limitat.

## Normes darrere dels valors per defecte {#standards-behind-the-defaults}

- **Mesura.** Sonoritat integrada i true peak tal com els defineix
  ITU-R BS.1770 (ponderació K, amb porta).
- **Objectiu −16 LUFS.** Segueix les recomanacions d'AES per a l'streaming i la
  distribució en línia. AES TD1008 (2021) substitueix l'anterior AES
  TD1004.1.15-10, que recomanava de −16 a −20 LUFS per a l'streaming. No és un
  valor de radiodifusió: EBU R128 utilitza −23 LUFS i ATSC A/85 utilitza
  −24 LKFS. Fes servir un d'aquests quan una especificació de lliurament ho
  exigeixi (EBU R128 per a l'intercanvi de programes de radiodifusió europeu,
  ATSC A/85 per a la televisió digital dels EUA), i −16 per a un flux o un
  programa en línia.
- **Sostre de true peak −1 dBTP.** El true peak màxim d'EBU R128, també
  utilitzat per les recomanacions d'AES. Els codificadors amb pèrdua poden
  sobrepassar en descodificar, de manera que un flux enviat per un codificador
  de baixa taxa de bits pot voler un sostre més baix com −2 dBTP (una pràctica
  habitual, que no forma part d'aquestes normes).
- **Guany màxim i ajust màxim, 12 dB.** Decisions de producte sense cap norma
  al darrere: suficients per pujar una gravació fluixa i per corregir una
  pista d'orella, no tant que es pugi el soroll o que l'ajust anul·li
  l'objectiu.
- **La normalització és activada per a les instal·lacions noves.**
  Actualitzar l'aplicació conserva l'ajust que ja té la teva configuració, de
  manera que una instal·lació que la tenia desactivada continua desactivada
  fins que l'actives.
- **L'objectiu de sonoritat del mesurador** (Configuració → Vúmetres) és un
  ajust a part, per defecte el valor R128 de −23 LUFS. La normalització no
  l'utilitza.
