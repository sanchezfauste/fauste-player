# Automatització

L'automatització manté l'emissora en antena quan no hi ha ningú a l'estudi.
Aquesta versió té el **musicalitzador**, un reproductor que tria la música per
si sol, a partir de les regles que fixes, i que mai no s'atura per manca de
llista, i la columna **Graella**, que executa els blocs horaris del dia, amb
l'editor on els crees.
Els reproductors que ja fas servir (P1 … Pn) no es toquen: continuen sent teus
per als programes que prepares i emets en directe.

## La columna Graella {#the-schedule-column}

La columna **Graella** reprodueix els **blocs** del dia: grups d'elements amb
una hora d'inici (un indicatiu horari, falques, un programa). Mostra-la amb el
mosaic **Graella** de la barra superior, igual que mostres un reproductor.

![La columna Graella en marxa, al costat de la columna Música: els blocs del dia per hora d'inici, cadascun amb el seu indicatiu de l'emissora (el rellotge el marca com a puntual), falques i la rotació que el musicalitzador reprodueix després; la cançó del musicalitzador en antena i les següents](../../images/guide/automation-columns.png)

La seva llista és la graella, agrupada per blocs: l'hora d'inici de cada bloc
i després els seus elements. Mostra avui, l'últim bloc d'ahir i el primer de
demà. Un bloc canviat per a un dia ho indica. L'element en antena i el següent
es marquen com en un reproductor; els elements que ja s'han emès apareixen
atenuats, i un element que no s'ha emès (**No emès**) apareix ratllat. Un
rellotge marca un element **puntual**, que només sona a la seva hora.

A sobre de la llista:

- **Inicia** posa en marxa la graella: cada bloc sona a la seva hora i el
  musicalitzador sona entre blocs. Res no es posa en marxa per si sol: després
  de cada inici de l'aplicació, l'automatització està aturada fins que prems
  Inicia.
- **Atura després de l'actual** deixa que acabi el que és en antena, i després
  ja no sona res per si sol.
- **En marxa** o **Aturada** diu en quin estat és.

Per canviar què sona a continuació, fes doble clic en un element, o fes clic
dret sobre seu i tria **Que soni a continuació**; segueix la cançó que és en
antena. **Torna al rellotge**, al mateix menú, deixa que la graella torni a
triar segons l'hora.

## Editar la graella {#editing-the-schedule}

**Edita la graella**, a la columna Graella, obre l'editor. Té tres pestanyes, i
**Rotacions…** et porta a Configuració → Automatització, on es creen les
rotacions.

![L'editor de graella, pestanya Plantilles: les plantilles Base i Weekend a l'esquerra; els blocs de Base a la dreta, cadascun amb el seu inici, els seus elements, Afegeix de categoria i la rotació de la música que va després](../../images/guide/schedule-templates.png)

- **Plantilles.** Una plantilla és un dia de blocs; una instal·lació nova en
  té una, "Base", per a tots els dies. Crea, duplica, canvia el nom o elimina
  plantilles a l'esquerra; els blocs de la triada són a la dreta. Escriu
  l'inici d'un bloc com a `HH:MM:SS` (una hora que no existeix es queda al camp
  amb una nota, i no canvia res). Arrossega pistes de la biblioteca a un bloc
  per afegir-les; **Afegeix de categoria** afegeix un element que reprodueix
  una pista d'aquella categoria, una de diferent cada vegada; **Música
  després** tria la rotació que el musicalitzador reprodueix després del bloc
  (la rotació per defecte si no se'n tria cap). Cada element es pot marcar com
  a **Puntual**, moure o treure. Quan els elements d'un bloc duren més que el
  temps fins al bloc següent, el bloc diu quant, en ambre; tot i així el pots
  deixar així.
- **Setmana tipus.** Quina plantilla fa servir cada dia de la setmana.

  ![La pestanya Setmana tipus: Base de dilluns a divendres, Weekend dissabte i diumenge](../../images/guide/schedule-week.png)

- **Dies.** Tria una data al calendari (les dates amb canvis són en ambre).
  Una data pot fer servir una plantilla diferent de la del seu dia de la
  setmana (un festiu en dijous pot fer servir la del cap de setmana). Els seus
  blocs es poden canviar només per a aquella data: el canvi queda marcat, i
  **Torna a la plantilla** el desfà. **Aquest dia no** treu un bloc només
  d'aquella data, i **Afegeix un bloc aquest dia** n'afegeix un. Els canvis
  posteriors a la plantilla continuen arribant als blocs que no vas canviar en
  aquella data.

  ![La pestanya Dies: el calendari amb avui triat i les dates amb canvis en ambre; el primer bloc d'avui té una falca més, marcat com a "canviat per a aquest dia", amb Torna a la plantilla](../../images/guide/schedule-days.png)

## El musicalitzador {#the-music-filler}

El musicalitzador és la columna **Música**. Està amagada fins que la mostres:
fes clic al mosaic **Música** de l'esquerra de la barra superior. Se situa a
l'esquerra de P1.

Té l'aspecte d'un reproductor i funciona com a tal: el rellotge gran, la forma
d'ona, els botons de transport, el fader i la llista de pistes. La diferència
és a la llista:

- **s'omple sola** a partir d'una **rotació** (més avall), sempre amb mitja
  hora d'avantatge (el *marge per endavant*, a Configuració →
  Automatització);
- **es retalla sola**: conserva les últimes 50 pistes que ha reproduït i treu
  les més antigues, de manera que pot funcionar durant setmanes;
- abans d'un bloc de la graella pot **canviar cançons que ha triat ella
  mateixa** per unes altres la durada de les quals càpiga en el temps que
  queda (vegeu més avall); les cançons que has afegit tu mai no es toquen;
- encara la pots **canviar a mà**: arrossegar pistes des de la biblioteca,
  reordenar files, treure una cançó que no vols. Una fila que afegeixes es
  marca amb una mà i mai no se substitueix; una fila que treus no compta com a
  reproduïda.

Les seves sortides es defineixen a Configuració → Sortides d'àudio, a la fila
**Música**, com per a qualsevol reproductor. No té número, ni dreceres de
teclat ni assignacions MIDI, i el control remot no la veu.

Prem **Play** per posar-la en marxa i **Atura després de l'actual** per deixar
que la cançó en antena acabi i s'aturi. Com tots els reproductors, mai no es
posa en marxa per si sola, ni tan sols després d'un reinici.

A sobre de la seva llista, on un reproductor té les pestanyes de les seves
llistes, la columna Música mostra la **rotació** que reprodueix. Tria'n una
altra allà: les cançons que tenia en cua se substitueixen i les que has afegit
es queden. Quan la rotació no troba res per reproduir (les seves categories
són buides, o falten tots els fitxers), la barra ho diu.

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
pas es salti alguns torns: amb Clàssics a 0–1, una cançó clàssica sona després
de l'indicatiu unes vegades i altres no. El màxim és sempre com a mínim 1. Una
rotació és la **per defecte**: una instal·lació nova en té una, **Música**, que
reprodueix tota la teva música.

Les rotacions es creen a Configuració → Automatització:

![Configuració, Automatització: les rotacions Música (la per defecte) i Hits, la Música seleccionada amb el seu únic pas sobre tota la música, les seves quantitats Com a mínim i Com a màxim, i les regles del musicalitzador](../../images/guide/settings-automation.png)

## Com tria {#how-it-chooses}

Dins d'un pas, el musicalitzador prefereix les pistes que porten més temps
sense emetre's, amb una mica d'atzar entre elles perquè l'ordre mai no es
converteixi en un cicle fix. També manté separats:

- **la mateixa pista:** pot tornar a sonar quan hagin passat 3 hores o quan
  hagi sonat des d'aleshores la meitat de les altres pistes del seu pas, el
  que passi primer;
- **el mateix artista:** no en 30 minuts.

Compta tot el que ha sortit en antena, des del musicalitzador, un reproductor
o la cartutxera; la preescolta no. Quan una categoria petita no pot complir les
regles, el musicalitzador cedeix abans que quedar-se en silenci: primer en
l'artista i després en la pista. Configuració → Automatització té tots els
valors, i és on crees, canvies el nom i modifiques les rotacions (vegeu
[Configuració](settings.md#automation)).

Abans d'un bloc de la graella, el musicalitzador també tria les seves últimes
cançons per la seva durada, perquè la música acabi netament: la cançó que fa un
fos de sortida quan comença el bloc perd com a màxim 30 segons, i no comença
cap cançó que sonés menys de 45 segons. Quan cap cançó de la rotació pot
complir les dues coses, sona la que menys perd. De moment, els dos valors només
es poden canviar al fitxer de configuració (`automation.max_fade_cut_secs` i
`automation.min_song_play_secs`).

Un **Indicatiu horari** és un tipus de pista propi (Biblioteca → Tipus ▸
Indicatiu horari), per a la identificació de l'emissora a l'hora en punt.
