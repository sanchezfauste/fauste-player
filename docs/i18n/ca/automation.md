# Automatització

L'automatització manté l'emissora en antena quan no hi ha ningú a l'estudi.
Aquesta versió té el **musicalitzador**, un reproductor que tria la música per
si sol, a partir de les regles que fixes, i que mai no s'atura per manca de
llista, i la columna **Graella**, que executa els blocs horaris del dia, fets
a partir de plantilles a Configuració.
Els reproductors que ja fas servir (P1 … Pn) no es toquen: continuen sent teus
per als programes que prepares i emets en directe.

## La columna Graella {#the-log-column}

La columna **Graella** reprodueix els **blocs** del dia: grups d'elements amb
una hora d'inici (un indicatiu horari, falques, un programa). Mostra-la amb el
mosaic **Graella** de la barra superior, igual que mostres un reproductor.

![La columna Graella en marxa, al costat de la columna Musicalitzador: els blocs del dia per hora d'inici, cadascun amb el seu indicatiu de l'emissora (el rellotge el marca com d'hora fixa), falques i la rotació que el musicalitzador reprodueix després; la cançó del musicalitzador en antena i les següents](../../images/guide/automation-columns.png)

La seva llista sembla una llista de reproducció, agrupada per blocs. Cada bloc
comença amb una fila de capçalera: la seva hora d'inici, quants elements té i
quant duren, "canviat per a aquest dia" quan l'has canviat avui i, en ambre,
quant es passa del bloc següent. Després venen els seus elements, amb l'**Hora**
a què està previst que comenci cadascun. La llista mostra avui, l'últim bloc
d'ahir i el primer de demà. L'element en antena i el següent es marquen com en
un reproductor; els elements que ja s'han emès apareixen atenuats, i un
element que no s'ha emès (**no emès**) apareix ratllat. Un rellotge marca un
element d'**hora fixa**, que només sona a la seva hora. Una **pista de
categoria** mostra la pista triada per a ella tan bon punt passa a ser la
següent (o ve després de la següent al seu bloc), amb la categoria al darrere:
aquesta és la pista que sona. Si la categoria no té cap pista que pugui sonar
(no en té cap d'assignada, o en falten els fitxers), la fila ho diu en ambre, i
l'element no s'emetrà. Un element el fitxer del qual resulta il·legible quan
està a punt de sonar se salta i també es marca com a no emès. Les pistes
s'assignen a una categoria a la biblioteca. La fila del **musicalitzador** diu
fins quan sona, i es mostra en antena mentre el musicalitzador sona per a la
Graella. Un següent l'hora del qual ja ha passat mostra l'hora en ambre.
Arrossega la línia entre dues capçaleres de columna per canviar l'amplada de
les columnes, com en una llista de reproducció; les finestres de plantilla i
de dia fan servir les mateixes amplades.

### Canviar la graella d'avui {#changing-todays-log}

La Graella es canvia com una llista de reproducció, i el canvi és només per a
aquell dia: la plantilla es queda com està.

- Els botons de la part inferior de la columna: **+ Pista…** obre la
  biblioteca, per arrossegar pistes a la llista; **+ Pista de categoria…** i
  **+ Musicalitzador (rotació)…** pregunten quina i l'afegeixen just a sota de
  la fila que has seleccionat (fes clic en una fila per seleccionar-la;
  selecciona una capçalera de bloc per afegir al començament d'aquell bloc),
  o, sense res seleccionat o a sota d'una fila que ja no pot canviar, al final
  del bloc que ve; **+ Bloc horari…** demana una hora, amb la primera lliure ja
  posada. Quan la columna és massa estreta per a tots, els que no hi caben són
  darrere de **⋯**.
- Arrossega pistes de la biblioteca, o fitxers d'àudio del teu gestor de
  fitxers, a una fila per posar-los davant seu (o darrere, a la meitat
  inferior de la fila, com en una llista de reproducció: així van al final
  d'un bloc), i arrossega files dins del mateix dia per moure-les. La fila que
  afegeixes queda seleccionada i a la vista, de manera que la següent que
  afegeixis va a sota.
- Fes clic dret en una fila per a **Hora fixa** i **Treu de la graella
  d'avui**. Fes clic (o clic dret) en una capçalera de bloc per afegir una
  **Pista…**, una **Pista de categoria…** o un **Musicalitzador (rotació)…** al
  final d'aquell bloc, o per a **Canvia l'hora…**, **Treu el bloc** i, en un
  bloc canviat, **Torna a la plantilla**.
- Mentre arrossegues sobre una capçalera de bloc, la línia de sota indica que
  ho deixes anar al començament d'aquell bloc; per posar files al final del
  bloc de sobre, deixa-les anar a la meitat inferior de la seva última fila.
- El que ja ha sonat, l'element en antena i els dies passats no es poden
  canviar. Un canvi al bloc en antena té efecte després de l'element en
  antena.

La columna Graella té la capçalera i els botons d'un reproductor, i funcionen
com en un reproductor:

- **Reproduir** reprodueix l'element marcat com a següent i posa en marxa
  l'automatització: cada bloc sona llavors a la seva hora, i el musicalitzador
  sona entre blocs. Quan el següent l'ha triat el rellotge i el seu bloc
  comença més tard, la música sona fins llavors. Res no es posa en marxa per si
  sol: després de cada inici de l'aplicació, l'automatització està aturada fins
  que prems Reproduir.
- **Pausa**, **Stop** i **Stop amb fos** actuen sobre el que és en antena per a
  la Graella (el seu element, o la cançó del musicalitzador entre blocs) i
  aturen l'automatització. Reproduir després de Pausa continua des d'on era;
  mentre està en pausa, el següent no es mou, i en reprendre el bloc següent
  manté la seva hora. Reproduir després d'un Stop durant la música torna a
  posar música, i el bloc següent comença a la seva hora.
- **Stop al final** deixa que acabi el que és en antena, i després
  l'automatització s'atura. Si sona la música i abans arriba l'hora d'un bloc,
  el bloc comença puntual i l'automatització s'atura després del seu primer
  element.
- **Pista anterior** no fa res a la Graella.
- **SINGLE** fa manual la Graella: Reproduir reprodueix l'element següent, i
  s'atura després d'ell; cap bloc no comença per si sol.

Mentre el musicalitzador sona entre blocs, la capçalera de la Graella diu
**Musicalitzador · *rotació*** (la rotació que reprodueix) i els seus botons
continuen funcionant; la cançó, el seu mesurador i la seva forma d'ona són
només a la columna Musicalitzador.

Quan l'automatització s'atura després d'un element, el següent queda en
l'element de sota del que ha sonat, i Reproduir el reprodueix, sigui l'hora que
sigui: tries tu. Quan s'atura durant la música, Reproduir torna a posar música
fins al bloc següent, tret que hagis triat un altre següent. Per canviar què
sona a continuació, fes doble clic en un element, o fes clic dret sobre seu i
tria **Que soni a continuació**. **Torna al rellotge**, al mateix menú, posa
com a següent el que diu l'hora.

## Plantilles i dies {#templates-and-days}

La Graella reprodueix cada dia a partir d'una **plantilla**: un dia de blocs.
Una instal·lació nova en té una, "Base", per a tots els dies. Les plantilles es
fan a Configuració → **Automatització**, a **Plantilles i dies** (no a
Android); la columna Graella només canvia el dia d'avui, com s'ha vist a dalt.

![Configuració, Automatització, Plantilles i dies: les plantilles Base i Weekend amb Edita…, Plantilla nova, Duplica, Elimina i el nom; la setmana tipus, Base de dilluns a divendres i Weekend dissabte i diumenge; el mes amb avui triat, les dates amb canvis en ambre, la plantilla de la data i Edita el dia…](../../images/guide/settings-plans.png)

- **Plantilles.** Tria'n una i prem **Edita…** (o fes-hi doble clic) per
  obrir-la; **Plantilla nova**, **Duplica** i **Elimina** són al costat, i el
  seu nom a sota.
- **Setmana tipus.** Quina plantilla fa servir cada dia de la setmana.
- **Dies.** Tria una data al calendari (les dates amb canvis són en ambre).
  Una data pot fer servir una plantilla diferent de la del seu dia de la
  setmana (un festiu en dijous pot fer servir la del cap de setmana).
  **Edita el dia…** obre aquella data, per canviar-ne els blocs només per a
  aquella data; les dates passades no es poden canviar.

Una plantilla o una data s'obre a la seva pròpia finestra, que tanca
Configuració. És la mateixa llista que la columna Graella, i es canvia igual:
els botons de la part inferior (Pista…, Pista de categoria…, Musicalitzador
(rotació)…, Bloc horari…, que afegeixen a sota de la fila que selecciones),
pistes arrossegades des de la biblioteca (o fitxers des del teu gestor de
fitxers) a una fila, files arrossegades per moure-les, el menú de clic dret de
les files i el menú de les capçaleres de bloc. Mai no sona, i pot quedar-se
oberta al costat de la biblioteca. La finestra d'avui mostra el que ha sonat i
el que sona, com la Graella; la finestra d'un dia es tanca a mitjanit, quan
aquell dia ja no pot canviar.

![Una plantilla oberta a la seva finestra: els blocs de la plantilla Base per hora d'inici, cadascun amb el seu indicatiu de l'emissora, les seves falques i el musicalitzador després, i Afegeix a la part inferior](../../images/guide/template-popup.png)

Una **pista de categoria** en una plantilla reprodueix una pista d'aquella
categoria, una de diferent cada vegada. L'element **Musicalitzador** tria la
rotació que el musicalitzador reprodueix després del bloc (la rotació per
defecte quan el bloc no en té cap). Quan els elements d'un bloc duren més que
el temps fins al bloc següent, la seva capçalera diu en quant, en ambre; tot i
així pots deixar-ho així.

Un canvi en una data queda marcat com a "canviat per a aquest dia" al seu bloc,
i **Torna a la plantilla**, al menú de la capçalera del bloc, el desfà. Els
canvis posteriors a la plantilla continuen arribant als blocs que no has
canviat en aquella data. Un canvi en una plantilla fet mentre un dels seus
blocs és en antena deixa aquell bloc com està per a avui (mostra "canviat per
a aquest dia"); el canvi li arriba a partir de l'endemà.

## El musicalitzador {#the-music-fill}

El musicalitzador és la columna **Musicalitzador**. Està amagada fins que la
mostres: fes clic al mosaic **Musicalitzador** de l'esquerra de la barra
superior. Se situa a l'esquerra de P1.

Té l'aspecte d'un reproductor i funciona com a tal: el rellotge gran, la forma
d'ona, els botons de transport, el fader i la llista de pistes. La diferència
és la llista:

- **s'omple sola** a partir d'una **rotació** (més avall), sempre amb una
  mitja hora d'avançament (el *marge per endavant*, a Configuració →
  Automatització);
- **es retalla sola**: conserva les últimes 50 pistes que ha reproduït i treu
  les més antigues, de manera que pot funcionar durant setmanes;
- abans d'un bloc de la graella pot **canviar cançons que ha triat ella
  mateixa** per unes altres la durada de les quals encaixi en el temps que
  queda (vegeu més avall); les cançons que has afegit tu no es toquen mai;
- encara pots **canviar-la a mà**: arrossegar pistes des de la biblioteca,
  reordenar files, treure una cançó que no vols. Una fila que afegeixes es
  marca amb una mà i no se substitueix mai; una fila que treus no compta com a
  reproduïda.

Les seves sortides s'ajusten a Configuració → Sortides d'àudio, a la fila
**Musicalitzador**, com per a qualsevol reproductor. No té número, ni dreceres
de teclat ni assignacions MIDI, i el control remot no la veu.

Prem **Reproduir** per posar-la en marxa i **Stop al final** per deixar que
acabi la cançó que és en antena i s'aturi. Com tots els reproductors, mai no
es posa en marxa per si sola, ni tan sols després d'un reinici. Mentre sona per
a la Graella entre blocs, aturar-la o posar-la en pausa també atura
l'automatització, i el seu Stop al final atura l'automatització quan acaba la
cançó; si abans arriba l'hora d'un bloc, el bloc sona i la marca es treu.
Després d'haver posat en pausa la Graella durant la música, Reproduir a la
columna Musicalitzador reprodueix la cançó pel seu compte: el Reproduir de la
Graella ja no la reprèn, i el seu Stop ja no l'atura.

A sobre de la llista, on un reproductor té les pestanyes de les seves llistes
de reproducció, la columna Musicalitzador mostra la **rotació** que
reprodueix. Tria'n una altra allà: les cançons que tenia a la cua se
substitueixen i les que has afegit es queden. Quan la rotació no troba res a
reproduir (les seves categories són buides, o falten tots els fitxers), la
barra ho diu.

## Rotacions {#rotations}

Una **rotació** és una llista curta de **passos** que el musicalitzador
reprodueix per torns, i després un altre cop des del primer. Cada pas indica
d'on surten les seves pistes (una categoria de la biblioteca amb les seves
subcategories, o totes les pistes d'un tipus) i quantes en reprodueix cada
vegada, com un interval:

| Pas | Reprodueix |
|---|---|
| Èxits actuals | 2–3 |
| Indicatius de l'emissora | 1 |
| Clàssics | 0–1 |

Amb un interval, un indicatiu de l'emissora arriba després de dues cançons una
vegada i després de tres la següent. Un interval des de **0** permet que un
pas se salti alguns torns: amb Clàssics a 0–1, una cançó clàssica sona després
de l'indicatiu unes vegades i d'altres no. El màxim sempre és com a mínim 1.
Una rotació és la **per defecte**: una instal·lació nova en té una, **Música**,
que reprodueix tota la teva música.

Un pas la categoria del qual no té cap pista que pugui sonar (una categoria
buida, o fitxers que falten) reprodueix en el seu torn una cançó de la rotació
per defecte (qualsevol música, si el pas és de la mateixa rotació per
defecte), de manera que els altres passos mantenen el seu lloc: un indicatiu de
l'emissora continua arribant entre cançons.

Les rotacions es creen a Configuració → Automatització:

![Configuració, Automatització: les rotacions Música (la per defecte) i Hits, la Música seleccionada amb el seu únic pas sobre tota la música, les seves quantitats Com a mínim i Com a màxim, i el començament de Plantilles i dies](../../images/guide/settings-automation.png)

## Com tria {#how-it-chooses}

Dins d'un pas, el musicalitzador prefereix les pistes que fa més temps que no
s'emeten, amb una mica d'atzar entre elles perquè l'ordre no es converteixi mai
en un cicle fix. També manté separats:

- **la mateixa pista:** pot tornar a sonar quan hagin passat 3 hores o quan
  hagi sonat des de llavors la meitat de les altres pistes del seu pas, el que
  passi primer;
- **el mateix artista:** no en 30 minuts.

Compta tot el que ha sortit en antena, des del musicalitzador, un reproductor o
la cartutxera; la preescolta no. Quan una categoria petita no pot complir les
regles, el musicalitzador cedeix abans que quedar-se en silenci: primer en
l'artista i després en la pista. Configuració → Automatització té tots els
valors, i és on crees, reanomenes i canvies les rotacions (vegeu
[Configuració](settings.md#automation)).

Abans d'un bloc de la graella, el musicalitzador també tria les seves últimes
cançons per la seva durada, perquè la música acabi netament: la cançó que baixa
amb un fos quan comença el bloc encara sona a l'hora del bloc i en perd com a
molt 30 segons després, i no comença cap cançó que hagués de sonar menys de 45
segons. També manté les regles d'artista i de pista sempre que pot. Quan cap
cançó de la rotació no pot complir les dues coses, sona la que menys en perd,
de manera que de vegades es poden sentir uns segons d'una cançó. Una pausa de
la música mou les cançons que van darrere, i el musicalitzador les torna a
triar. El musicalitzador també evita les cançons, i els artistes, que un bloc
reprodueix poc després. De moment, els dos valors només es poden canviar al
fitxer de configuració (`automation.max_fade_cut_secs` i
`automation.min_song_play_secs`).

Un **Indicatiu horari** és un tipus de pista propi (Biblioteca → Tipus ▸
Indicatiu horari), per a la identificació de l'emissora a l'hora en punt.
