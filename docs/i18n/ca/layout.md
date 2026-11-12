# Disposició

Organitza la finestra principal per al teu torn: dona als reproductors i a la
cartutxera les mides que vulguis, posa la cartutxera sota els reproductors o al
costat, amaga els panells que no facis servir i maximitza'n un per un moment.
La disposició es desa amb la sessió i torna a l'inici següent; una instal·lació
nova comença amb tots els reproductors de la mateixa amplada i la cartutxera
sota ells.

## Mostrar i amagar panells {#show-and-hide-panels}

Els mosaics de l'esquerra de la barra superior, **Graella** i **Música** (la
[graella](automation.md#the-schedule-column) i el
[musicalitzador](automation.md#the-music-filler), tots dos amagats fins que
els mostres), **P1** … **Pn** i **Cartutxera**, mostren quins panells hi ha a
la pantalla: un mosaic il·luminat és mostrat. Fes clic en un mosaic per
amagar o mostrar el seu panell. Amb molts reproductors, els mosaics es
desplacen lateralment (fes girar la roda del ratolí sobre ells) i el botó de
disposició continua a la vista. Els altres panells ocupen l'espai que deixa,
i un reproductor que es torna a mostrar torna a la seva amplada.

Amagar només treu un panell de la pantalla. Un reproductor amagat continua
sonant, i la seva tecla numèrica, el MIDI, l'API remota, la seva finestra CUE
i l'automatització encara el controlen. Un mosaic mostra l'estat del seu
panell, tant si el panell és mostrat com amagat: un punt vermell mentre un
reproductor sona (o la cartutxera mentre sona un cartutx) i dues barres
ambre, com un botó de Pausa, mentre un reproductor és en pausa.

Amagar un reproductor n'esborra la pista seleccionada, de manera que **Supr**
no pugui treure una pista que no veus. Els fitxers deixats anar a la finestra
lluny d'una llista van a l'últim reproductor que has fet servir, o al primer
reproductor mostrat; amb tots els reproductors amagats es refusen, i la barra
d'estat ho diu.

El mosaic **Biblioteca**, després d'ells, obre i tanca la [finestra de la
biblioteca](library.md), que flota sobre els panells; no és un panell, de
manera que la regla de l'últim mosaic il·luminat de més avall no s'hi aplica.
No es mostra en un mòbil.

Almenys un panell es queda mostrat: l'últim mosaic il·luminat no es pot
clicar. Mentre la cartutxera és amagada, la drecera que la plega o la desplega
no fa res; fes clic a **Cartutxera** per mostrar-la.

## Canviar la mida dels reproductors {#resize-the-players}

Arrossega l'espai entre dos reproductors per moure amplada d'un a l'altre.
Els altres reproductors no es mouen, i cap reproductor és més estret del que
necessiten els seus controls. Fes doble clic a l'espai per tornar a donar a
tots els reproductors la mateixa amplada.

Quan la finestra és massa estreta per a tots els reproductors mostrats a la
seva mida mínima, la fila dels reproductors es desplaça lateralment, com ha
fet sempre.

## Moure i canviar la mida de la cartutxera {#move-and-resize-the-cartwall}

La cartutxera es col·loca sota els reproductors, o a l'esquerra o a la dreta
de la finestra a tota l'alçada: tria **Cartutxera a baix**, **Cartutxera a
l'esquerra** o **Cartutxera a la dreta** al [menú de disposició](#the-layout-menu).
Sota els reproductors és tan alta com necessiten les seves files, com a màxim
sis desens de l'alçada entre la barra superior i la barra d'estat; al costat
d'ells ocupa al principi tres desens de l'amplada.

La cartutxera mai és més estreta que la seva capçalera (**Aturar-ho tot**, el
seu títol i una pestanya de pàgina) ni que les seves columnes de botons a la
seva mida mínima. Quan la finestra és massa estreta per a ella i per a tots
els reproductors mostrats a la seva mida mínima, la fila dels reproductors es
desplaça lateralment.

Arrossega l'espai entre la cartutxera i els reproductors per fer-la més gran
o més petita: amunt o avall quan és sota els reproductors, lateralment quan és
al costat. Els reproductors conserven l'espai que necessiten els seus
controls. Fes doble clic a l'espai per tornar a la mida inicial: tan alta com
necessiten les seves files sota els reproductors, tres desens de l'amplada al
costat. La mida es desa amb la disposició; moure la cartutxera la torna a
començar a la seva mida inicial.

Quan canvies la mida de la cartutxera, o mentre és al costat dels
reproductors, els seus botons creixen per omplir-la; els noms i els temps
conserven la seva mida, al mig de cada botó. Un nom es divideix entre les
paraules; una paraula massa llarga per al seu botó acaba amb punts
suspensius (…) en lloc de trencar-se entre línies. Quan la cartutxera és més
petita del que necessiten les seves files i columnes amb els botons més
petits, es desplaça.

Fes clic a **CARTUTXERA** per plegar la cartutxera. Sota els reproductors només
en queda la capçalera; al costat esdevé una franja estreta de l'alçada de la
finestra, amb una fletxa per tornar-la a desplegar i, mentre sona un cartutx,
un punt vermell. Desplegada, recupera la mida. Mentre és plegada, l'espai
al seu costat no fa res.

## Maximitzar un panell {#maximize-a-panel}

La capçalera de cada reproductor, i la de la cartutxera, acaba amb dos botons
petits: **Amagar** (un signe menys) i **Maximitzar** (cantonades que apunten
cap a fora; **Restaurar**, cantonades que apunten cap a dins, al panell
maximitzat). Passa-hi el cursor per veure'n el nom. Són atenuats mentre la
disposició és bloquejada, **Amagar** a l'últim panell mostrat, i **Maximitzar**
amb un sol panell mostrat i en una cartutxera plegada. Les mateixes dues
accions són al menú d'un clic dret al fons de la capçalera (no en un dels seus
botons):

- **Maximitzar** dona a aquell panell tot l'espai: els altres panells
  s'encongeixen a la seva mida mínima. És només per un moment i no es desa.
  **Restaurar** al mateix menú hi posa fi, i també ho fan amagar o mostrar un
  panell, bloquejar la disposició, moure la cartutxera, canviar el nombre de
  reproductors i arrossegar o fer doble clic en un espai entre panells; les
  mides d'abans tornen. Una cartutxera plegada no es pot maximitzar, i
  plegar una cartutxera maximitzada hi posa fi.
- **Amagar** amaga el panell, com fa el seu mosaic de la barra superior.

## El menú de disposició {#the-layout-menu}

El botó de disposició, a la dreta dels mosaics, obre un menú amb:

- **Cartutxera a baix**, **Cartutxera a l'esquerra**, **Cartutxera a la
  dreta**: on es col·loca la cartutxera;
- **Bloquejar la disposició**: mentre és marcada, res canvia la disposició per
  accident: els mosaics, els altres elements d'aquest menú, els espais entre
  els panells i **Maximitzar** i **Amagar** de les capçaleres no fan res. La
  reproducció, i plegar o desplegar la cartutxera, funcionen com sempre. El
  bloqueig es desa amb la disposició;
- **Igualar l'amplada dels reproductors**: tots els reproductors de la mateixa
  amplada;
- **Restablir la disposició**: torna a la disposició d'una instal·lació nova.
